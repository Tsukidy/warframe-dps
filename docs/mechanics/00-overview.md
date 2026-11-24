Status: reviewed (spec)

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
- [Review notes](#review-notes)

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
arch-gun rules cover (the Necramech exalted weapons Ironbride and Arquebex are out, `01` Table 4.3-A), archmelee
(Archwing melee) weapons (`07` OQ-737), flat damage additions applied after the base bucket (Ballistic Battery, Volt
passive, Energy Channel, decrees such as Fragile Vanity and Storm Caller; no stage of E-1 to E-11 can express a
post-A per-type flat, `01` OQ-120), server-only features of the inspiration (CLAUDE.md). Multi-target effects (AoE radius on other
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
| Body-part multipliers, head-crit capability, non-head weak points and crit promotion: prose (`Multis`) plus the `W:Enemy_Body_Parts` tables (flags † non-headshot weak point, * non-headcrit weak point, ¹ crit promotion; `05` §4.3, OQ-530) | parsed per enemy from both; `docs/data/` adds the tables as a source; `02` and `05` define defaults |

---

## 2. Notation and glossary

### 2.1 Units and numeric conventions

| Rule | Definition |
|---|---|
| U-1 Fractions | Inside the engine every chance, bonus and percentage is a **fraction**: 0.30 means 30 %. Percent appears only at the UI and data-import boundary. Status and crit chances may exceed 1. |
| U-2 Bonus vs multiplier | A *bonus* is an additive fraction summed inside a bucket (`sum_base_dmg = 1.65` for Serration). A *multiplier* is a factor (`1 + sum_base_dmg = 2.65`). Symbols starting with `sum_` are bonuses; symbols ending with `_mult` are factors. |
| U-3 Units | time in seconds, distance in metres, damage and hit points in HP, rates per second, armor in armor points. Event times are compared on the integer microsecond grid of CD-14. |
| U-4 Precision | f64 everywhere except the emulation points NP-1 and NP-2 below, where the game's binary32 arithmetic changes results. |
| NP-1 | Base critical multiplier quantization (E-4) in f32 (`W:Critical_Hit` §Quantization, `F:core` §9). |
| NP-2 | Enemy level-scaling curves and their smoothstep blend in f32, evaluated as `f1 + (f2 − f1)·s` (`F:enemy` §7.1). Riven values in f32 (`F:mods` §8.3) belong to `04` (E-4.18); `09` uses them for range validation. DoT ticks are f64 (Decision: the f32 tick rule of `W:Damage/Calculation` §DoT changes results by < 1e-6 relative). |
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
| Family | A firing attack (no parent) together with all its descendants (children, lingering children, reload-triggered attacks of the same weapon per `06` 9.5); `08` E-8.23. In game a family hits one target and its members see each other's procs (T-22, CD-12). |
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
| `str_base_mult` | m_str | factor | `Str·R_r` for model-1 exalted weapons, whose base damage scales with Strength (`R_r = s_r/s_3` by exalted ability rank, 1 at rank 3, T-31; `01` E-1.1); else 1 |
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
| `cc_base`, `cc_base_add` | CC0, CC0+ | fraction | attack base crit chance; flat additions before mods (Incarnon "+X % base") |
| `sum_cc_rel` | ΣCC_rel | bonus | relative crit chance (Point Strike, Blood Rush and Gladiator via combo, weak-point crit chance on weak-point hits, shards, zoom) |
| `sum_cc_abs` | ΣCC_abs | fraction | absolute crit chance added after mods (Arcane Avenger, Cat's Eye, Covenant, Piercing Navigator since U40, Puncture `punc_cc`, flat-crit toggle) (`W:Piercing_Navigator` L27-28, L38) |
| `cc_floor` | CC_min | fraction | lower bound (Smeeta Charm 2.0), 0 by default |
| `cc` | CC | fraction | final crit chance (E-5) |
| `cd_base`, `cd_base_add` | CDM0, CDM0+ | factor | attack base crit multiplier; additions before quantization (Incarnon "+X base"; the conditional Prelude of Might addition CDM0c is `02` E-2.5) |
| `cd_q` | Q(·) | factor | quantized base crit multiplier (E-4) |
| `sum_cd_rel` | ΣCD_rel | bonus | relative crit damage (Vital Sense, shards) |
| `sum_cd_abs` | ΣCD_abs | factor | absolute crit damage added after mods (Arcane Crepuscular, Tenacious Bond, Spiteful Defilement, Cold `cold_cd`) |
| `cd_field_mult` | M_cdf | factor | buffing-field crit damage multiplier (Electric Shield ×2), Decision: multiplies CDM before tiers (UNVERIFIED, OQ-016) |
| `cd` | CDM | factor | final crit multiplier (E-4) |
| `crit_tier` | k | integer ≥ 0 | rolled tier (E-6) |
| `tier_raise_chance` | p_vig | fraction | Vigilante set chance to raise a crit by one tier |
| `headcrit` | h | factor | part crit factor from `05` data: 1 by default, 2 on head-crit heads, other values per enemy (Juggernaut maw and belly 4, Ropalolyst head 0.2, `Multis` 1.5; `W:Enemy_Body_Parts` L46); radial instances use the body part's h, or the parent instance's part for `radial_parent_part` attacks (`02` E-2.8) |
| `part_mult` | P_part | factor | enemy body-part multiplier of the part hit (P_body is 1 for most enemies, Eidolon bodies 2, armor plates 0.5; 3 on most heads) |
| `head_wp_weight` | k_wp | 1.5 or 1.0 | weight of weak-point damage bonuses on headshot parts / other weak points |
| `sum_wp_dmg` | ΣWP | bonus | weak-point damage bonuses (Acuity, Seek, riven) |
| `sum_hs_mult` | ΣHS | bonus | headshot-multiplier bonuses (Deadhead, Target Acquired, zoom); headshot parts only ("Effects that specify headshots do not apply against non-headshot weak points", `W:Enemy_Body_Parts` L134) |
| `sonar_mult` | M_son | factor | Sonar and Detect Vulnerability weak-spot multipliers (location and nodes T-32; inherited by DoT, E-13) |
| `part` | Part | factor | `Part_A·Part_B·M_son` (E-7, `02` E-2.10; ΣBP and M_hsu are `02` symbols) |
| `sc_base`, `sc_base_add` | SC0, SC0+ | fraction | base status chance; Incarnon flat additions before mods |
| `sum_sc_rel`, `sum_sc_abs` | ΣSC_rel, ΣSC_abs | bonus, fraction | relative and absolute status chance |
| `sc_mult` | M_sc | factor | status-chance multipliers (Tau `1 + 0.1·n_TAU`, custom "vulnerable to status chance") |
| `sc` | SC | fraction per pellet | final status chance (E-8) |
| `proc_weight[s]` | w_s | fraction | proc-type probability (E-9) |
| `forced_procs[s]` | φ_s | count per pellet | forced procs (weapon traits, stance hits via `07`) |
| `p_hunter`, `p_ib` | p_HM, p_IB | fraction | Hunter Munitions 0.30 per crit instance (k ≥ 1); Internal Bleeding / Hemorrhage 0.35 (×2 when the modded FR < 2.5, strict) once per instance that carries ≥ 1 Impact status and no own (rolled or forced) Slash proc; Magnetic Welt likewise for Magnetic (`02` E-2.15) |
| `ms_base`, `sum_ms`, `ms` | MS0, ΣMS, MS | factor | multishot (E-10) |

**Status effects (owner `03`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `sum_status_dmg` | A_sd | bonus | status-damage bucket (Elementalist, riven, Galvanized Elementalist) |
| `status_dmg_type[s]` | A_sd,s | bonus | per-status additions in the same bucket (Emerald shard Toxin, Conductive Sphere Electricity) |
| `sum_status_dur` | SD | bonus | status duration |
| `status_dur_type[s]` | SD_s | bonus | per-type status duration of procs of s, additive with SD (Ash's passive SLA +0.50, `W:Ash/Abilities` L3-6; `03` §5) |
| `dot_coeff[s]` | K_s | factor | 0.35 SLA; 0.5 HEA, TOX, ELE, GAS; 0.3 BLA per stack |
| `dot_elem[s]` | E^dot_s | bonus | elemental bonus scaling the DoT of s (`03` §9.3, the owner: for PE every elemental bonus of that element, combined or not (mods, riven stats, arcanes, archon shards, Valence Formation, non-combining additions), buffing-field elements and Nourish excluded; only literal Gas bonuses for GAS; none for SLA, BLA); computed as `elem_dot_sum[s]` (`01` E-1.15, section 7.2) |
| `dur_base[s]`, `dur[s]` | D0_s, D_s | s | base and modded duration `D0_s·(1 + SD + SD_s)` |
| `dot_ticks[s]` | N_s | count | ticks per proc `floor(6·(1 + SD + SD_s) + ε)`, ε = 1e-9 (`03` E-3.1) |
| `dot_delay[s]` | δ_s | s | time of the first tick after the proc (1 for SLA, HEA, TOX; 0 for ELE, GAS) |
| `stack_cap[s]` | cap_s | count | per-type cap after enemy overrides |
| `stacks[s]` | n_s | count | live stacks on the target |
| `viral_mult(n)`, `mag_mult(n)` | Vir(n), Mag(n) | factor | `2 + 0.25(n − 1)` for n ≥ 1, else 1 |
| `corr_strip(n)` | c_corr(n) | fraction | `min(1, 0.20 + 0.06 n)` for n ≥ 1, else 0 |
| `heat_strip(k_h)` | c_heat | fraction | `1 − 0.5^(k_h/4)`, k_h ∈ {0 .. 4} the target's Heat strip step counter `heat_step` (`03` E-3.8): it ramps up after ignition, steps down over 6 s after the episode ends with no Heat live, and resumes from its step on re-ignition (`W:Damage/Heat_Damage` L96-98); 0 only at k_h = 0 (Heat-immune targets never leave 0) |
| `cold_cd(n)`, `punc_cc(n)` | – | factor, fraction | `0.10 + 0.05(n − 1)` (n ≤ 9), 1.0 when Frozen, 0.50 at 10 stacks on targets that cannot freeze (UNVERIFIED, `03` E-3.11); `0.05·n` for n ≤ cap_PUN (7 with Evensong, 0.35; `03` E-3.2, `W:Evensong` L17) |
| `n_distinct` | n | count | distinct status types on the target that count for CO-type bonuses |

**Conditional bonuses and buffs (owner `04`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `sum_co` | C | bonus | CO-type bucket: Σ (per-status bonus × n) + per-stack terms (Secondary Shiver per Cold stack) + conditional terms |
| `co_behavior` | – | enum | `adding`, `multiplying`, `none` for this attack (catalogue) |
| `co_ratio` | ρ_co | factor | catalogue ratio `B_co / B_ref` (1.0 by default; 0.5 bows at full charge; > 1 child projectiles; 0.2 on the multiplying Tenet Agendus heavy waves) |
| `co_ms_factor` | κ | factor | CO effectiveness on projectiles created by multishot (pellets j ≥ 1): Tenet Detron 0.65, else 1 (`04` §2.4) |
| `co_base_abs` | B_co^abs | HP | fixed CO base of `co_ref = fixed` catalogue rows (Noctua, Artemis Bow, Sporelacer; `04` §2.6) |
| `mult_final_omitted` | G_F | factor | product of final multipliers the Adding-CO recalculation omits (Eclipse, Garuda passive, Parasitic Link, Overdriver, Duality, Furious Javelin, Hall of Mirrors, Longbow Sharpshot, Primary Compression on projectiles, Extinguished Dragon Key, range falloff, Exalted Blade wave falloff Φ_w; bow charge scaling is 1 for every data attack, because charged and partial shots are separate data attacks, `04` §2.7, §3.1) |
| `mult_other` | G | factor | product of every other independent weapon-side multiplier (melee combo M(c) as M_h, Slam Damage, ability combo M_ab, sniper combo, Primed/Charged Chamber, Synth Charge, beam ramp, Vauban passive, Navigator, Mecha Empowered, Melee Careen ...). The stance hit multiplier S_j and the heavy-attack multiplier H_j are not G members: they are ST-1 base factors for every CO behaviour (`07` E-7.20, E-7.20b), so the Incarnon flat is not multiplied by H_j (`W:Melee` L681) |
| `extra_hit_frac` | p_xh | fraction | extra-hit fraction (Xata 0.26 × Str, Toxic Lash 0.30 × Str, ×2 melee) |
| `stealth_bonus` | SDB | bonus | `1 + 0.2·min(rank, 30)` for weapons with maximum rank 30 (`W:Stealth` L44-46); `1 + 0.15·rank` for weapons with maximum rank 40 (Paracesis, Kuva and Tenet melee; `W:Finisher` L103-104); 7 (+700 %) at maximum rank in both; used as `1 + SDB` in c_k |
| `vuln[ℓ][t]` | V_ℓ,t | factor | target damage vulnerability on layer ℓ for type t: `Π(1 + v_mult)·(1 + Σv_add)` over the entries whose layer and type masks match (`05` E-5.19; default layer mask {HP} UNVERIFIED, `05` OQ-523); catalogue in `04`, stage in `05`. `V` without indices means the entry for the instance's layer and type |

**Enemies (owner `05`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `level`, `base_level`, `level_q` | L, L0, q | level | `q = L − L0` for health, shield, armor; overguard uses `L − 1` (D-042) |
| `steel_path`, `eximus` | – | bool | modifiers of `05` section 7 (E-5.4, E-5.5) |
| `pool[ℓ]` | OG, SH, HP | HP | scaled pools |
| `armor_init` | A0 | armor | scaled armor, `floor`ed, clamped to [200, 2700] when base armor > 0 |
| `armor_eff` | A_eff | armor | A0 after strips (E-15) |
| `armor_dr(A)` | DR(A) | fraction | `0.9·sqrt(A/2700)` for A ≤ 2700, else `A/(A + 300)` |
| `type_mod[f][t]` | φ_t | factor | faction damage-type modifier (1.5, 1, 0.5); on OG only Void 1.5 |
| `innate_dr[ℓ]`, `status_dr` | – | fraction | enemy flat damage reduction per layer and on status damage; with class and part DR they form `dr_total[ℓ]` R_ℓ (`05` E-5.20) |
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
| `stance_hit_mult` | S_j | factor | stance multiplier of hit j of a combo; an ST-1 base factor (`07` E-7.20), like the heavy multiplier H_j (E-7.20b) |

**Evaluation (owner `08`)**

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `t_win` | T_win | s | fight window, default 20 (T-01) |
| `t_ttk_max` | T_ttk | s | TTK horizon, default 60 |
| `proc_rate[s]` | λ_s | 1/s | expected procs of s per second |
| `stack_mean[s]` | μ_s | count | mean of the rolled live-proc count before capping, as seen by a consumer (section 4.2: in steady state the long-run mean at the consumer's arrival, not the time average λ_s × D_s) |
| `n_runs`, `seed` | – | count, u64 | Monte Carlo run count (default 1000) and seed |

---

## 3. Master per-hit formulas

### 3.1 Stage pipeline

Every damage instance passes through these stages in this order. Commutative multiplications may be implemented
in any order; non-commutative steps (quantization, the additive CO term, additive stealth term, armor floor of 1,
attenuation) MUST keep the stated position.

| Stage | Name | Owner | Produces |
|---|---|---|---|
| ST-1 | Base vector | `01` (inputs from `06` kitgun, `07` zaw/stance; stance and heavy multipliers S_j, H_j from `07` E-7.20, E-7.20b) | `base[t]`, `base_total`, `base_ref` |
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
| CO-omitted final multipliers `mult_final_omitted` (G_F) | each a separate factor | Eclipse, Garuda passive, Parasitic Link, Overdriver, Duality, Furious Javelin, Hall of Mirrors, Longbow Sharpshot, Primary Compression (projectile), Extinguished Dragon Key, range falloff, Exalted Blade wave falloff Φ_w; bow charge scaling (1 for every data attack, `04` §2.7) | ST-4 | `04` (falloff `06`, Φ_w `07`) | `F:addmul` §1.6 |
| Faction `sum_faction` (F) | additive | bane mods, Roar, Damage Blessing, riven faction, Paracesis Sentient bonus | ST-5; again on DoT ticks and extra hits | `01` | `F:core` §6, `F:addmul` §3 |
| Other multipliers `mult_other` (G) | each a separate factor | melee combo multiplier M(c) (M_h), ability combo M_ab, sniper combo, Primed/Charged Chamber (additive with each other), Synth Charge, beam ramp, Vauban passive, Navigator, Mecha Empowered, Melee Careen, Slam Damage mods (M_slam), finisher bucket, Braton Incarnon multishot-pellet damage. Not members: the stance hit multiplier S_j and the heavy-attack multiplier H_j, which are ST-1 base factors for every CO behaviour (`07` E-7.20, E-7.20b; `W:Melee` L681: the Incarnon flat "is not multiplied by the Heavy Attack Multiplier, but is multiplied by the combo multiplier") | ST-5 | `04` (`06`, `07` for timing-driven ones) | `F:addmul` §0, §12, `F:weapon` §16 |
| Melee projectiles: universal weapon damage | additive; a second base bucket multiplied with A | Arcane Arachne, Vigorous Swap, Holster Amp, Toxin Ward, Power Throw; Vex Armor, Deathbringer, Amp, Rift Torrent count in both buckets | ST-4 (exception for glaive throws and explosions, Exodia Contagion) | `04` | `F:addmul` §12 |
| Body part (P_part, ΣWP, ΣHS, ΣBP, M_hsu, M_son) | ΣWP additive, ΣHS additive (headshot parts only), ΣBP additive (non-headshot hits), factors multiply | Acuity, Seek, riven WP; Deadhead, Target Acquired, zoom; Meticulous Aim body penalty; Cernos Prime unique headshot factor; Sonar, Detect Vulnerability | ST-6 | `02` | `F:core` §10, `02` E-2.10 |
| Crit chance (ΣCC_rel, ΣCC_abs, CC_min, M_cc) | additive within each; M_cc multiplies | E-5 (M_cc: custom p.d, Deadly Maneuvers) | ST-7 | `02` | `F:core` §8 |
| Crit damage (ΣCD_rel, ΣCD_abs, M_cdf) | additive within each | E-4 | ST-7 | `02` | `F:core` §9 |
| Multishot (ΣMS) | additive | Split Chamber family, Galvanized multishot, Metronome Opera, Primary Frostbite (base additions MS0+: Strun and Torid Incarnon) | E-10 | `02` | `F:core` §7 |
| Status chance (ΣSC_rel, ΣSC_abs, M_sc) | additive within each; M_sc multiplies | status mods, Weeping Wounds via combo, shards; Tau | E-8 | `02` | `F:core` §13 |
| Status damage (A_sd, A_sd,s) | additive | Elementalist mods, riven Status Damage, Galvanized Elementalist, Emerald shard (Toxin), Conductive Sphere (Electricity), Ash passive (Slash) | E-13 only | `03` | `F:status` §3.1 |
| Status-damage vulnerability (V_sd) | additive group, snapshotted at proc creation | Burning Hate, Pageflight's status-damage increase, custom p.o (grouping of p.o UNVERIFIED, `03` OQ-324) | E-13 only | `03` (values `04`) | `W:Burning_Hate` L28-34 |
| Status duration (SD, SD_s) | additive; SD_s adds for procs of s only | Lasting Sting family, Hunter Track, Saryn passive (every proc of the build, D-037); SD_s: Ash passive (Slash +0.50) | E-13, CD-06 | `03` | `F:status` §2.3, `03` §5 |
| Enemy side (φ, Vir, Mag, V, DR, R_ℓ, Att) | φ, Vir, Mag, V, DR and R_ℓ multiply; V has an additive subgroup | faction type modifiers; Viral; Magnetic; Molecular Prime and other vulnerabilities; armor; non-armor DR R_ℓ (innate, class, status, part DR; `05` E-5.20); attenuation | ST-8 | `05` (vulnerability sources `04`) | `F:enemy`, `F:buffs` §2 |

### 3.2 Weapon-side stats (E-1 to E-10)

```text
E-1  Base vector (ST-1)
     B'_t = B0_t·m_str + V_t + X_t            m_str = Str·R_r for model-1 exalted weapons, else 1 (01 E-1.1)
     V_t  = v·Σ B0  if t = e_v, else 0         B' = Σ_t B'_t ;  B_ref = Σ_t (B0_t·m_str + V_t)
     melee strikes: S_j (light kinds) or H_j (synthesized heavies) multiply the base inside ST-1 (07 E-7.20, E-7.20b;
     H_j does not multiply X_t)
E-2  Composition (ST-2)
     r_t = (B'_t / B')·(1 + P_t)·(1 + S^phys_t)          t ∈ IPS
     r_t = B'_t / B' + E_t                               t elemental (innate value of t joins; combined elements per 01;
                                                         for a combined element B'_t is the summed innate base of its
                                                         group, 01 E-1.9)
     r_t = B'_t / B'                                     t ∈ {VOI, TAU, TRU, FIN}
E-3  Quantization (ST-3), per type, once per final type
     q_t = sign(r_t)·floor(|r_t|·32 + 0.5) / 32
E-4  Crit multiplier (02 E-2.5)
     x    = CDM0 + CDM0+ + cond·CDM0c          (f32 sum; cond = [CC < θ_P], Prelude of Might, strict)
     code = floor(clamp(x, 0, 32)·4095/32 + 0.5);  Q = code·32/4095   (f32, NP-1)
     CDM  = (Q·(1 + ΣCD_rel) + ΣCD_abs)·M_cdf
E-5  Crit chance (02 E-2.4)
     CC   = max(CC_min, ((CC0 + CC0+)·(1 + ΣCC_rel) + ΣCC_abs)·M_cc);   CC_r = max(0, CC)
E-6  Crit tier (02 E-2.6, E-2.7)
     k = floor(CC_r) + Bernoulli(frac(CC_r));  if k ≥ 1: k += Bernoulli(p_vig);  then k += π_p (Critical Promotion part)
     E[k] = CC_r + p_vig·min(1, CC_r) + π_p
E-7  Body part (02 E-2.10)
     direct hit on part p (p = the T-03 part; the body part for an attack flagged `torso_only`, 02 §4.10, Peacemaker):
                            Part = Part_A · Part_B · M_son
       Part_A = P_p + k_wp·ΣWP     on a weak point (k_wp = 1.5 on headshot parts, 1.0 on other weak points; a weak
                                   spot of Sonar or Detect Vulnerability is a weak point); 1 there for 1×-headshot attacks
              = P_p                otherwise
       Part_B = (1 + ΣHS)·M_hsu    on a part with the headshot flag
              = 1                  on a `non_body` part (05 §4.3: neither headshot nor bodyshot, so no ΣHS, M_hsu or
                                   ΣBP; W:Enemy_Body_Parts L37)
              = 1 − ΣBP            otherwise (Meticulous Aim bodyshot penalty)
     radial instance:  Part = P_r·(1 − ΣBP)·M_son,  r = the body part, or for an attack flagged `radial_parent_part`
                       (02 §4.10; Zymos "Headshot Explosion", 06 §9.6) the part hit by its parent instance; never a
                       headshot or weak point (M_son only from a spot that reaches radial instances, 02 E-2.10;
                       ΣBP on radial UNVERIFIED, 02 OQ-228)
E-8  Status chance (per pellet)
     SC = ((SC0 + SC0+)·(1 + ΣSC_rel) + ΣSC_abs)·M_sc
E-9  Proc count and type (per pellet)
     procs = floor(SC) + Bernoulli(frac(SC)) + Σ_s φ_s          (UNVERIFIED sampling rule; E[procs] is sourced)
     w_s   = q_s·[s not immune] / Σ_u q_u·[u not immune]
E-10 Multishot (02 E-2.1)
     MS = max(1, (MS0 + MS0+)·(1 + ΣMS));  Acuity mods: MS = MS0;  pellets = floor(MS) + Bernoulli(frac(MS))
     child attacks (parent ≠ none, 06 E-6.23): no multishot draw; their instances per parent shot are 06 E-6.24
```

Sources: E-1 `F:core` §1, §3.1, §3.2, `F:weapon` §14, `W:Melee` L681 (heavy multiplier and Incarnon flat); E-2
`F:core` §4, §5, `F:melee` §2.1 (stance physical multiplicative); E-3 `F:core` §14, `W:Damage/Calculation`
§Quantization, `W:Damage` patch history (1/32 since U40); E-4 `F:core` §9, `W:Critical_Hit` §Quantization and
§Critical Damage, Cold placement `F:status` §4.5, `M_cdf` `F:buffs` §1.12 (placement UNVERIFIED), Prelude of Might
`W:Braton_Incarnon_Genesis` (via `02` E-2.5); E-5 `F:core` §8, Smeeta floor `F:buffs` §8, M_cc `W:Deadly_Maneuvers`
L30 ("multiplicatively applied after buffs"; its position inside the CC_min floor follows `02`, the owner); E-6
`F:core` §9, Vigilante `F:mods` §7.2, Critical Promotion `W:Enemy_Body_Parts` §Critical Promotion; E-7 `F:core` §10,
`W:Enemy_Body_Parts` (L134: "Effects that specify headshots do not apply against non-headshot weak points"; L46: part
crit factors; L37: `non_body` hitboxes are "neither bodyshot nor headshot locations (no effect from Meticulous Aim)"),
`W:Meticulous_Aim` ("Bodyshot damage penalty is multiplicative to weapon overall damage"), `W:Sonar` (torso spots
raise AoE damage), `W:Peacemaker` L84 (`torso_only`: "targets the enemy's torso"), `W:Zymos` L34
(`radial_parent_part`: the explosion is "affected by headshot and headcrit bonuses"); E-8 `F:core` §13, Tau `F:status` §4.15; E-9 `F:status` §2.1, §2.2 (quantized weights:
Decision, D-024); E-10 `F:core` §7, `W:Multishot` (floor at 1 for every weapon: D-019), `F:weapon` §10.1 (children).

### 3.3 The base scalar Y and the Condition-Overload family (ST-4)

The CO family (Condition Overload, Galvanized Aptitude/Savvy/Shot, Secondary Shiver, Cedo, Incarnon per-status
perks, Shattering Frost) sums into `C` (`F:addmul` §1.1). Its behaviour is a property of each **attack**, taken from
the catalogue owned by `04` (`W:Condition_Overload_(Mechanic)` §Attack Catalog; default `adding` with
`ρ_co = 1`). The scalar Y that multiplies the quantized composition is:

```text
E-11 Base scalar
  Y0          = B'·max(0, 1 + A)          (01 E-1.11; melee projectiles: the per-factor clamped two-bucket form of 04 §3.4)
  none        : Y = Y0·G_F
  multiplying : Y = Y0·(1 + ρ_co·C)·G_F    ρ_co = 1 except catalogue rows (Tenet Agendus heavy waves 0.2)
  adding      : Y = Y0·G_F  +  κ·B_co·C    B_co = ρ_co·B_ref, or the catalogue's fixed B_co^abs (co_ref = fixed, 04 §2.6)
                                           κ = co_ms_factor on pellets j ≥ 1 of a shot (Tenet Detron 0.65), else 1
  then        : Y ← max(0, Y)              (Decision, UNVERIFIED, 01 OQ-106)
```

- `none` applies to slams and other radial melee explosions, radial children of **projectile** parents (rocket,
  grenade and embedded explosions), standalone AoE attacks and lingering-damage attacks (`ShotType = DoT`), the
  catalogue's "does not apply" rows, and every attack whenever the conditions toggle is `none` (T-02). Other radial
  and cloud damage follows the catalogue or the defaults D1-D6 of `04` §2.5: a hitscan explosion is `adding` on the
  directly hit target only (from the hitscan damage), beam AoE is `adding` on every target (Glaxion Vandal), an
  embedded cloud gives CO on the embedded target only (default `adding`; catalogue rows override it, e.g. Torid
  "Poison Cloud" `multiplying`, Pox "Poison Cloud" `adding` per tick), and non-radial child projectiles of projectile
  parents are `adding` with the parent as reference (`W:Condition_Overload_(Mechanic)` §Mechanics, §Application
  Behaviors).
- In `adding`, the CO term is not multiplied by `(1 + A)` or `G_F` and uses `B_co` (no Incarnon flat). Everything after
  ST-4 (faction, G, body part, crit, enemy side) multiplies Y as a whole, including the CO term. That last point is the
  wiki's framing of every example but is stated on no page: Decision, UNVERIFIED, `OQ-001`.
- `multiplying` uses `B'` including the Incarnon flat (Kunai Incarnon evidence), and `G_F` multiplies the whole. The
  ratio on multiplying rows and κ follow the catalogue: Tenet Agendus heavy waves (catalogue row "Heavy Attack Wave 1":
  base 2800, CO base 560, 20 %, Multiplying; wave 2 3600 / 720 / 20 %), Tenet Detron "Only 65% effective on projectiles created by Multishot"
  (`W:Condition_Overload_(Mechanic)` cache L548-561, L437; `04` TV-04-34, TV-04-35).
- C is a scalar. It enters after quantization, so it does not change `q_t`, and the CO part is multiplied by the
  quantized composition: sourced by the Zhuge Prime catalogue row ("the 'real' in-game value is 48.4375 for both the
  base damage and aptitude damage" = 50 × 31/32, `W:Condition_Overload_(Mechanic)` L371; `OQ-002`).
- n (the distinct-status count inside C) counts statuses present on the target **before** this damage instance;
  statuses applied by earlier pellets of the same shot count, the instance's own procs do not (`F:addmul` §1.2).
  How n is obtained is T-05. Which earlier instances count is `04` E-4.2 (the rule only); the EV expectation of the
  per-instance counts n_j and n_COL,j is `08` E-8.10, the single owner, under the decisions of section 4.2.
- Melee stance inputs carry a per-input CO effectiveness ratio r_in (`F:addmul` §1.10); `07` owns the data (E-7.21),
  `04` the formula (E-4.6). S (S_j, or H_j on heavies) is part of the ST-1 base (`07` E-7.20, E-7.20b):
  `B'_j = S_j·B'` (heavies `H_j·(B' − X) + X`), `B_ref,j = S_j·B_ref` (heavies `H_j·B_ref`). The formula is E-11 with
  `B' := B'_j` and `B_ref := B_ref,j`, the `adding` CO term multiplied by r_in:
  `Y_j = B'_j·max(0, 1 + A)·G_F + r_in·κ·B_co,j·C`, `B_co,j = ρ_co·B_ref,j` (`co_ref = fixed` rows: `B_co^abs`), so S
  is applied once to each term (light strike, X = 0, not fixed: `B'·S_j·max(0, 1 + A)·G_F + r_in·κ·S_j·ρ_co·B_ref·C`);
  `multiplying` and `none` take no r_in. Whether r_in applies per hit or per input is UNVERIFIED (`04` OQ-401).

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
  Enemy side (ST-8), per type t, on layer ℓ, with target state σ just before the instance (05 E-5.14 to E-5.21):
    OG : D_t = W_t · [t = VOI ? 1.5 : 1] · Mag(n_MAG) · V_OG,t · M_OG · R_OG
    SH : D_t = W_t · φ_t · Mag(n_MAG) · V_SH,t · R_SH               (t ∉ B_SH)
    HP : D_t = W_t · φ_t · Vir(n_VIR) · V_HP,t · arm_t, then the minimum of 1, then × R_HP
         arm_t = 1 − DR(A_eff)  for armor-reduced types when A_eff > 0, else 1   (TRU, FIN, CIN bypass)
         after armor, every type with D_t > 0 is raised to at least 1      (armor floor; not after other DR)
    R_ℓ (SH, HP) = innate, class, status (status instances only) and part DR, multiplied (05 E-5.20);
    R_OG = innate and class DR only (no status DR on overguard)
    then D = Att(Σ_t D_t) on the layers listed in the enemy's attenuation data (05 E-5.21; never on OG)
  Layer routing (ST-9): OG absorbs everything first; then SH absorbs every type not in the bypass set B_SH (default
    {TOX, TRU}; Hounds {TRU}, Treasurer ∅; 05 E-5.15); types in B_SH go to HP while SH > 0, and HP takes everything
    once SH is gone. Overflow at a layer break: CD-08.
```

Sources: `F:core` §15, `F:enemy` §1-§6, §9, `F:status` §4.11, §4.13, `F:buffs` §2 (vulnerability, owner `05`),
`F:addmul` §9 (stealth adds to the crit term), `W:Stealth` §Stealth Damage Bonus (formula lines 37-49), `W:Finisher`
L103-104 (rank-40 weapons), `W:Damage_Reduction` L53, L101 (non-armor DR multiplies on health and shields, not on
overguard), True damage `Bypass = {"Armor", "Shield"}` in `Module:DamageTypes/data` (`05` E-5.15).

`M_OG` is the overguard-only weapon multiplier (Secondary Fortifier ×8, `F:buffs` §7). `G` contains every weapon-side
multiplier not in `G_F`; `04` owns the membership list and the rule that an unlisted multiplier defaults to `G`.

```text
E-15 Effective armor (05 E-5.10)
  A0    = clamp(floor(A_base · M_armor(q)), 200, 2700) if A_base > 0, else 0
  A_tot = max(0, A0 · f_flat · max(0, 1 − Σ ability_strips − Σ CP·aura_strength − T-07) − P_pts)
  A_eff = A_tot · (1 − c_heat) · (1 − c_corr(n_COR)),      c_heat = heat_strip(k_h(τ))
          f_flat: flat base-armor strips (Shattering Impact, Metal Auger), `max(0, B_A − s_flat)/B_A`;
          P_pts: point strips (Polarize; order UNVERIFIED, 05 OQ-507)
          (strips act on the clamped value and may take it below 200)
```

Sources: `F:enemy` §3.2-§3.3, `W:Armor` (U32 total-armor strips, U36 clamp), `W:Corrosive_Projection`,
`W:Damage/Heat_Damage`, `W:Damage/Corrosive_Damage`; flat and point strips `05` §9.3-§9.4.

### 3.5 Status DoT tick (E-13)

A DoT instance is created by a proc p of s ∈ {SLA, HEA, TOX, ELE, GAS} (and a BLA stack) on an inducing instance
with crit tier k and body part Part. Weapon-side terms are snapshotted at proc creation; target-side terms are
evaluated when each tick lands (CD-05), except the status-damage vulnerability V_sd, which is snapshotted (CD-05).

```text
E-13 DoT tick (03 E-3.4, E-3.5)
  seed_p    = Y · (1 + F) · G · b · c_k · Part_dot     (Y with the inducing attack's CO behaviour, 04 E-4.7; not
                                                         quantized; b = merged beam count of a beam tick, else 1)
  Part_dot  = Part                                     (Sonar spots are inherited since the U44 rework, D-095;
                                                         Detect Vulnerability spots by analogy, UNVERIFIED, 03 OQ-312)
  X_p       = K_s · (1 + E^dot_s) · (1 + F_tick,p) · (1 + A_sd + A_sd,s) · V_sd,p
  tick_𝒢    = ( Σ_{p∈𝒢} seed_p·X_p  +  a·X_{p₀} ) · M_tick(τ)        a = 1, the "+1" accumulator (D-025);
                                                                       p₀ = the oldest proc of the tick group 𝒢
              single proc:  (seed_p + 1) · X_p · M_tick(τ)
  M_dot,s   = V_sd,p · M_tick(τ):  V_sd,p = status-damage vulnerability live on the target when p is created (Burning
              Hate, Pageflight, custom p.o; snapshotted, an exception to CD-05); M_tick(τ) = target-condition second
              dips read at tick time (Mecha Empowered on a marked target, Vauban passive on an incapacitated target)
              (03 §9.4); the Magnetic-break proc takes neither (03 §13)
  F_tick,p  = F of the inducing instance, except Heat procs that join an episode (Heat inherit, 03 §10.3)
  Enemy side at tick time τ:
    SLA (Cinematic): no φ, no armor;  HP: × Vir(n_VIR(τ)) · V_HP · R_HP;  SH: × Mag(n_MAG(τ)) · V_SH · R_SH;
                     OG: × Mag · V_OG · M_OG,p · R_OG
    HEA, TOX, ELE, GAS, BLA: as E-12 with type s, M_OG,p (the source attack's M_OG) on overguard (TOX ticks bypass
                     SH, not OG)
  Ticks: N_s = floor(6·(1 + SD + SD_s) + ε), ε = 1e-9, at τ = t_proc + δ_s + m, m = 0 .. N_s − 1      (CD-06)
  Tick groups 𝒢 (03 E-3.5): SLA, TOX one proc each; ELE, GAS the instances whose ticks land at the identical time
         (one damage number, one "+1"); HEA the episode pool
  Beams: b in the seed, so DoT scales twice with multishot (CD-10)
  HEA: one shared timer per target; all live Heat procs are summed into one tick per second, "+1" added once (03)
  BLA: no "+1"; each stack deals 0.3·H at fuse expiry 1.5·(1 + SD) s, or at detonation (10 stacks, 4 on bosses) (03)
```

The crit multiplier is inside the stored seed, so the "+1" is not scaled by it: the source of the "+1" rule gives
"six critical seeds of 80.0195389" (= 40 × Q(2.0)) and "separate Slash stacks with seeds of 40 and 80.0195389 deal
(40 + 1) × 0.35 = 14.35 and (80.0195389 + 1) × 0.35 = 28.3568382", with M "the product of the remaining applicable
modifiers, such as elemental, faction, and status-damage bonuses" (`W:Damage/Calculation` cache L619-635; `03`
OQ-301). Putting `Part_dot` inside the seed as well follows the same structure by analogy (UNVERIFIED, `03` OQ-301).
Sonar inheritance: `W:Sonar` L25-26 ("DoTs ... will have their damage calculated on the resulting damage if proc'd
from a hit that hits a Sonar weakspot"; revision 2813083 of 2026-09-25: "All status DoTs are affected by Sonar now")
is the newer per-mechanic statement; `W:Damage_over_Time` L171 and `W:Damage/Slash_Damage` L60 keep the pre-rework
statement (the DoT page's only later edit, revision 2817738 of 2026-10-12, changed one Thermal Transfer line). Rule 3
of section 1.4 picks `W:Sonar` (D-095).

Sources: `F:status` §3.1-§3.4 (tick formula, inheritance list, "+1" accumulator UNVERIFIED single editor,
`W:Damage/Calculation` §DoT), §4.4 (Heat), §4.8 (Blast), `W:Damage_over_Time` §DoT Damage Scaling, faction double
dip `W:Faction_Damage_Bonus`, `W:Burning_Hate` L28-34 (V_sd snapshot), `W:Mecha_Empowered` L32-33 and
`W:Vauban/Abilities/Passive` L13-17 (second dips), `W:Secondary_Fortifier` L37-39 ("DoTs will also steal Overguard";
M_OG on status instances UNVERIFIED, `03` OQ-327), `W:Ash/Abilities` L3-6 (SD_s), `03` TV-03-13 (ε). Worked check:
`W:Damage/Slash_Damage` example `0.35 × 344.5 × 1.3 × 1.9 = 297.82` is E-13 without the "+1" (`F:status` §3.2).

### 3.6 Extra hit (E-14)

Xata's Whisper, Toxic Lash, Resupply, Silken Stride, Demonium, Sangodae, Gastroparesis (`F:addmul` §13, `F:buffs`
§1.3, §1.4, §1.22). One extra hit per pellet of a parent direct or radial instance on the target (and one per merged
beam tick, CD-10); the hits of explode-on-death and proc-made attacks on the target (Acid Shells, Concealed
Explosives, Nightwatch Napalm) and slam radial damage trigger too, each with its own damage as H_w. DoT ticks never
trigger; Bursting Mass's absorbed damage and Pathocyst's maggots never trigger. None if the parent kills the target;
no combo or sniper-combo gain; does not change the parent's elements or proc weights.

```text
E-14 Extra hit
  H_w   = Σ_t W_t(k)                                  (parent weapon-side damage, after quantization, crit, Part)
  XH    = H_w · p_xh · (1 + F) · (Xata ? Part : 1)    one damage instance of type e_xh (VOI for Xata, TOX for Toxic Lash)
  Enemy side: E-12 with the single type e_xh (Xata: Void φ, armor applies)
  Status: Xata rolls VOI with the weapon's SC; Toxic Lash and the others force one proc of e_xh
  Forced DoT of an extra hit (03 §9.8, the single definition; 04 E-4.14): seed S_p = XH (no "+1", UNVERIFIED);
    X_p  = K_e · (1 + E^dot_all,e) · (1 + F) · (1 + A_sd + A_sd,e) · V_sd,p   (E-13's X_p for type e; V_sd,p snapshotted
           when the proc is created)
    tick = S_p · X_p · M_tick(τ), × M_OG,p when it lands on overguard; enemy side as E-13
    E^dot_all,e = elem_dot_sum[e] (01 E-1.15): every elemental source of e on the weapon whose dot_scaling is true,
                  combined into a secondary element or not; buffing-field elements excluded (03 §9.3)
    a forced Heat proc joins the target's Heat episode; the Heat inherit rule then overrides its tick-level
                  elemental and faction factors (03 §10.1, §10.3; OQ-332 of 03)
  Melee Duplicate is not an extra hit of this form: it is a full copy of the hit with its own rolls (04)
```

Faction therefore applies twice on extra hits and three times on their DoT; Xata also applies the body part twice.
Sources: `W:Extra_Hit` §Damage Calculation and §Interactions L48-49 ("Most non-standard weapon hits will trigger an
Extra Hit ... The damage calculation will use the damage dealt by the non-standard hit as the Weapon Hit Damage";
Bursting Mass and Pathocyst exceptions), `W:Xata's_Whisper`, `W:Toxic_Lash` L47 ("This includes the radial damage of
slam attacks, explode on death mods such as Acid Shells, and mods like Concealed Explosives and Nightwatch Napalm ...
especially useful for ... (Kuva) Ogris") (via `F:addmul` §13, `F:buffs` §1.3-§1.4; `04` §6.2, D-412).

### 3.7 Cross-cutting decisions

| ID | Decision | Source |
|---|---|---|
| CD-01 | Fractions inside the engine (U-1). | brief |
| CD-02 | Quantization: only the composition ratios `r_t` are quantized, on the 1/32 grid, with the modded base as scale. Base bucket, CO, faction, G, G_F, body part, crit and all enemy-side terms are outside. DoT seeds and extra hits are not quantized again. | `F:core` §14 ("Get Scale → Quantize → Add → Apply all other multipliers"; the page's UpdateMe conflict yields identical numbers for uniform multipliers) |
| CD-03 | Each status roll and each crit roll is independent; crit and status of one pellet are independent. | `F:core` §7, §13 |
| CD-04 | Order inside one damage instance: (1) damage is computed from the target state just before it; (2) crit and procs are rolled; (3) damage is applied; (4) procs are applied. Pellets of one shot are sequential for the CO count n, but do **not** see each other's debuffs (Viral, Magnetic, Corrosive, Heat strip, Cold, Puncture). Shots of an instant burst are separate shots. Electricity and Gas tick 0 lands after step (4), so it sees a Viral proc of the same hit. | `F:status` §4.13 (Viral timing, Simulacrum-tested), `F:addmul` §1.2; extension to the other debuffs is UNVERIFIED (`OQ-028`) |
| CD-05 | Weapon-side terms of a DoT are snapshotted at proc creation; target-side terms (Viral, Magnetic, armor, type modifier, vulnerability, M_tick second dips) are evaluated at each tick. Exception: the status-damage vulnerability V_sd (Burning Hate, Pageflight, custom p.o) is snapshotted at proc creation ("only applies to status procs dealt after the vulnerability is applied"). | `F:status` §3.1, `W:Damage/Viral_Damage`, `W:Burning_Hate` L28-34 (`03` §9.4) |
| CD-06 | DoT tick count `floor(6·(1 + SD + SD_s) + ε)` with ε = 1e-9 (a guard against f64 summation error, part of the rule); debuff durations `D0·(1 + SD + SD_s)` without rounding; Heat strip interval `0.5·(1 + SD)`; Blast fuse `1.5·(1 + SD)`; negative SD rules in `03`. | `W:Lasting_Sting` §Notes; `03` E-3.1, TV-03-13; others UNVERIFIED (`OQ-005`) |
| CD-07 | Capped statuses (COL, COR, GAS, MAG, RAD, VIR, TAU, IMP, PUN) use independent timers; a new stack beyond the cap replaces the oldest. Enemy caps are `05` data (§4.4) applied by `03` E-3.2: status-capped enemies, 4 for every status (Kuva Liches, with Impact 5: the U27.3 override 6 cut to the global Impact cap 5 since U33.6, `03` §6.1; Acolytes, with Impact 3; Archons in Archon Hunts; Sisters of Parvos and Technocyte Coda, membership UNVERIFIED, `03` OQ-320); other bosses (`boss = true`) Cold 4 (no Frozen) and Blast 4 only, every other status at its normal cap (UNVERIFIED, `05` OQ-531); overguard present: Cold 4; VIP with overguard: Puncture 3; Evensong: Puncture 7 (`03` E-3.2). Application to DoT instances UNVERIFIED (`OQ-008`). | `F:status` §1, §2.4; U27.3 note `W:Bosses` L925 (Kuva Liches only; "with the exception of Impact which can stack up to 6 times"); `W:Damage/Impact_Damage` L22, L325-330 (Impact cap 5 since U33.6); `W:Acolytes` L17; `W:Archon_Boreal` L43; `W:Damage/Cold_Damage` L27, L364; `W:Damage/Calculation` L643-645 (Blast); `W:Overguard` L70 (U36, "Cold procs (up to 4 stacks)"); `W:Evensong` L17 |
| CD-08 | Layer order OG → SH → HP. Overflow of the instance that breaks OG is discarded. Overflow of the instance that breaks SH reaches HP at 5 % during the 0.1 s gate, at 100 % for weak-point hits; status ticks are not gated. | `F:enemy` §4.3, §5; overflow rules UNVERIFIED (`OQ-011`) |
| CD-09 | Attenuation and flat enemy DR apply per damage instance after every modifier. | `F:enemy` §9.1 (U34 note "Modifiers now apply before Damage Attenuation") |
| CD-10 | Non-beam: every pellet is its own damage instance. Beams: the beams of one tick that hit the target merge into one instance with summed damage and summed SC and a single crit roll; forced procs apply once per merged tick. | `F:core` §7, `W:Multishot` §Continuous Weapons |
| CD-11 | Single target by default (section 1.3). | brief |
| CD-12 | A family (a firing attack and its descendants, section 2.3) is one evaluation unit by default in the game profile: its members share one target status timeline merged by time, so a parent's procs (Viral, Corrosive, Heat strip, the CO count) act on its children and back; per-attack outputs are the members' shares (`08` E-8.23). Evaluating each attack on its own is the parity setting (T-22 off, compat profile; `I:agg` §7.5). Two firing attacks of one weapon keep independent schedules when grouped (no game source for alternation; `06` OQ-628). | `I:feat` §10; `F:weapon` §10.1; `08` §9 |
| CD-13 | Expected values: crit, multishot and proc counts enter linearly; target-state functions use the stack-count distribution (section 4.2), never `f(E[n])`. A quantity that exists only on crits (Hunter Munitions Slash) carries the crit-conditional mass C1, not E[c] (section 4.2). | section 4.2; `02` E-2.8, E-2.16 |
| CD-14 | Event times: both evaluators hold every schedule time, offset and duration on the integer microsecond grid of `08` E-8.0a (`τ̂ = round_half_even(τ·10^6)`); every equality, ordering, expiry and window test is an exact integer comparison on that grid. `03` ("equality is exact") and `06` (E-6.11, ε = 1e-9) state their time comparisons on it. | `08` §4.3, OQ-807 (consistency decision; no game evidence involved) |

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
Q = 256 × 32/4095 = 2.0004885 (f32, 2.000488519668579); CDM = Q × 2.20 = **4.4010747** (4.401074743270875; every
later step uses full precision). MS = **1.90** (1 pellet with
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

| Type | non-crit (k = 0) | yellow crit (k = 1, × 4.4010747) |
|---|---|---|
| IMP | 4.884688 | 21.497876 |
| PUN | 17.910522 | 78.825545 |
| SLA | 30.936356 | 136.153214 |
| COR | 141.655945 | 623.438402 |
| Σ | **195.387510** | **859.915037** |

E[c] = 1 + 0.30 × 3.4010747 = 2.0203224 → E[damage per pellet] = **394.7458**, per shot × 1.9 = **750.0170**.

**Step 6, Slash DoT from one pellet (03).** Non-crit seed = 35 × 2.65 × 1.55 = 143.7625; tick = (143.7625 + 1) ×
0.35 × 1.55 = **78.533656** (77.991156 without the "+1"). At k = 1 the crit multiplier is inside the seed (E-13):
tick = (143.7625 × 4.4010747 + 1) × 0.35 × 1.55 = **343.787408** (the superseded form, "+1" scaled by c_k, gave
345.632491). Cinematic: no φ, no armor, lands on health as is. Six ticks at t + 1 ... t + 6 s.

**Step 7, Xata's Whisper 0.26 on the same pellet (04).** XH = 404.332031 × c_k × 0.26 × 1.55 = **162.945809**
(k = 0), **717.136683** (k = 1). Void, φ = 1, armor applies: **59.055875** and **259.909320** on armored health.

**Step 8, proc rates (02).** w = q/Σq: IMP 0.022222, PUN 0.122222, SLA 0.211111, COR 0.644444. Pellets per second
while firing = 1.9 × 15.328 = 29.1232. Procs per second: IMP 0.168267, PUN 0.925471, SLA 1.598540, COR 4.879754.
No forced procs, so every live count is the random part of section 4.2 in the form of `08` E-8.3 (exact P(N = 0),
mean μ, zero-truncated Poisson shape); here it matches Poisson(μ) to 1e-4 (PUN: P(N = 0) 8.435e-5 against
e^−μ = 9.729e-5; P(n = 5) 0.952557 against 0.952548; E[min(n, 5)] 4.928436 against 4.928377), so the rounded values
below are unchanged. Expected procs per shot η = 1.9 × 0.26 × w: COR
0.318356, PUN 0.060378. Steady mean seen by a shot (section 4.2, one shot every 1/15.328 s): COR 0.318356 × 122 earlier
shots within 8 s = 38.84 live procs (the time average λ·D would be 39.04), P(n_COR = 10) > 0.99999999; PUN 0.060378 ×
153 shots within 10 s = 9.24, P(n_PUN = 5) = 0.953, E[min(n, 5)] = 4.928.

**Step 9, steady-state target (state B, stacks taken at their caps for illustration).** 10 COR, 5 PUN:
A_eff = 1355 × (1 − 0.80) = 271 (strips may go below the 200 floor), DR = 0.285132; CC = 0.30 + 5 × 0.05 = 0.55;
per type non-crit 9.634807, 35.327626, 61.020445, 279.409406 (Σ **385.392284**); E[c] = 1 + 0.55 × 3.4010747 =
2.8705911; E[damage per pellet] = **1106.3037**.

**Step 10, EV DPS (08).** State A, direct only: burst = 394.7458 × 29.1232 = **11,496.26**; sustained × ρ =
**7,986.83**. State B: direct burst 32,219.10; Slash DoT = 1.598540 procs/s × 6 ticks × (143.7625 × 2.8705911 + 1)
× 0.35 × 1.55 = **2,152.50** (Little's law; the seed is linear in c_k, so E[c] enters it directly; the inducing
hit's crit includes the Puncture bonus, UNVERIFIED `OQ-017`); burst total **34,371.60**; sustained **23,879.07**. A
crude steady-state TTK, HP / sustained = 73,621.375 / 23,879.07 = **3.08 s**, ignores the stack ramp; the EV window
TTK of `08` is longer.

---

## 4. Evaluation model

### 4.1 Common semantics

- **Scenario.** A scenario is (build, attack or attack group, enemy and its options, toggles, buffs, `t_win`). Both
  evaluators read the same resolved stats (stages ST-1 to ST-7) and the same deterministic **schedule generator**
  (`06` ranged, `07` melee), which lists shot times, reload intervals, charge delays, burst spacing, spool, beam ramp
  and melee hit multipliers. The schedule never depends on random draws.
- **Fresh state.** Every window starts with: no statuses on the target (except external statuses of T-06 and
  T-43), full magazine (or Incarnon charge magazine), combo counter at Initial Combo, sniper combo 0, beam ramp at its
  start value, spool at its start, conditional buffs per T-02, Galvanized and arcane stacks per T-02, gauge-gated
  attacks with a full kill gauge (`06` §6.7).
- **Layer views.** For the four views `health`, `armor`, `shield`, `overguard`, every damage instance is evaluated
  with that view's layer rules (E-12, E-13) against a target that never depletes. `health` ignores armor ("100 %
  armor stripped", the inspiration's Health panel). The status state is the same in every view, except `og_present`,
  which sets the overguard-dependent caps (Cold 4, hence no Frozen, and VIP Puncture 3, CD-07) and the overguard
  ragdoll immunity (`03` E-3.13; ρ_RAG = 0 in the `overguard` view, `08` E-8.27): the `overguard` view uses
  `og_present = true`, and the `health`, `armor` and `shield` views, which model damage as if overguard were gone
  (`05` §18), use `false`. On an enemy with overguard this gives two view timelines that differ only in these caps
  and in ragdoll immunity;
  the EV evaluator likewise evaluates each view with its own `og_present`. The `layered` view depletes OG → SH → HP,
  uses the live state (OG > 0 at the moment of application) and is used for TTK only. Source of the game rule:
  "Bosses, as well as enemies with active Overguard, can receive a maximum of 4 Cold stacks"
  (`W:Damage/Cold_Damage` L27; U36); the per-view split is a modelling
  Decision (`03` OQ-321, `05` OQ-529, `08` OQ-804).
- **DoT accounting.** *Landed*: a tick counts at the time it lands; ticks after `t_win` do not count toward window
  metrics and are reported as "pending DoT". *Credited*: the whole DoT of a proc counts at the proc time (the
  inspiration's accounting, `I:agg` §6, `JS L10982-10986`); used only for parity outputs. Steady-state metrics use
  per-proc totals (equal to landed in the long run, Little's law).
- **DPS definitions** (all per attack and per view):

| Output ID | Definition |
|---|---|
| `dps_burst` | Expected damage per second during uninterrupted fire (no reload, unlimited ammo) with every stacking state at its steady-state distribution (section 4.2), beam ramp full, spool full, and combo, sniper combo and conditional stacks at their steady value (section 4.2). DoT enters as Σ_s λ_s × (expected total damage of one proc of s). Heat: see section 4.2. |
| `dps_sustained` | As `dps_burst` over full magazine cycles: direct damage and proc rates scaled by the duty ρ (`06` defines ρ per weapon class, e.g. bows, battery, per-shell, infinite ammo); stack distributions use the cycle-average proc rate (approximation). Spool weapons use the per-magazine spool-up (ρ and T_cyc from the walked schedule, spool reset on reload, `06` OQ-603); beams use the mean ramp factor η_rmp over the ticks of a steady magazine (magazine 2 onward; the ramp decays after 0.8 s idle, `W:Continuous_Weapon` L19), which scales damage but not proc rates (`06` §10.3-§10.4; UNVERIFIED metric choice, `06` OQ-620). For weapons without reload (melee, infinite ammo) it equals `dps_burst`. |
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
| Crit tier | linear: use `E[c] = c_0 + E[k]·(h·CDM − 1)` (c_0 = 1, or 1 + SDB on a melee stealth hit; exact for any CC; Vigilante and Critical Promotion via E-6). DoTs and extra hits inherit the inducing crit linearly, so their expectations multiply by the same `E[c]` (crit and procs are independent, CD-03), except procs that exist only on crit instances (Hunter Munitions; any on-crit forced proc): their DoT mass uses `C1 = E[c·1{k ≥ 1}] = E[c] − P(k = 0)·c_0` (`02` E-2.8, E-2.16; exact by probability: `02` TV-02-10 gives E[c·X_SLA] 0.5151 against E[c]·E[X_SLA] 0.3714). A weapon-side factor g(k) that depends on the tier uses `E[c·g(k)]` (`02` E-2.19). |
| Multishot | linear: expected pellets `MS` per shot. Effects that depend on pellet order (the sequential CO count) use, per instance j, the presence probabilities of the instances before j weighted by their existence probabilities (pellet `floor(MS)` exists with probability `frac(MS)`); row "Distinct statuses" below, `08` E-8.10. |
| Proc counts | expected procs of s per pellet `SC·w_s + φ_s` plus the proc-triggered procs of `02` E-2.15 to E-2.18; proc rate `λ_s` per second. Hunter Munitions `p_HM·P_c` per instance (P_c = P(k ≥ 1)); Internal Bleeding / Hemorrhage `p_IB·π_IB` once per instance, `π_IB = P(Impact status ∧ no own Slash)` (`02` E-2.16; Magnetic Welt likewise with π_W); the pair gives at most one Slash: `E[X_SLA] = 1 − (1 − p_HM·P_c)(1 − p_IB·π_IB)`, crit-weighted mass `E[c·X_SLA] = p_IB·π_IB·E[c] + p_HM·C1·(1 − p_IB·π_IB)` (`F:addmul` §8; `W:Internal_Bleeding`, `W:Hemorrhage`, `W:Magnetic_Welt` §Notes: "Proccing impact more than once ... will not allow this mod to proc more than once", "Cannot produce multiple procs in a single instance of damage alongside any other Slash sources"; `W:Hunter_Munitions`). |
| Live stacks | the live count of s at τ is (procs certain to exist, i.e. forced procs of instances that exist with probability 1, whose duration window covers τ) + a random part with mean μ_s (the expected rolled and other random procs whose duration window covers τ): Poisson(μ_s) when certain procs are present, otherwise exact probability of 0 Π_e (1 − A_{s,e}) and a zero-truncated Poisson positive part that keeps the mean (`08` E-8.3, A-3); independent across status types; stacks `n_s = min(N, cap_s)`. Per event, the presence of s is the exact `P(event applies ≥ 1 proc of s)` of `02` E-2.18 (A_s per instance), not `1 − exp(−η)` (a forced proc is present with probability 1, which Poisson(1) would put at 0.632; `08` G-1, OQ-801; `08` §5.3 combines the exact presence with the count distribution). Every nonlinear target function uses this distribution: `E[Vir(n)]`, `E[Mag(n)]`, `E[DR(A_eff)]` over the joint distribution of n_COR and the Heat strip step k_h (`08` E-8.6), `E[cold_cd(n)]`, `E[punc_cc(n)]`. |
| Distinct statuses | `E[n] = Σ_s P(n_s ≥ 1) + external statuses (T-06, T-43)`; `P(n_s ≥ 1) = 1 − Π_e (1 − A_{s,e})` over the events e whose procs of s cover τ (= 1 when a certain forced proc covers τ; `1 − exp(−μ_s)` for a count that is Poisson only). Exact by linearity under the independence assumption. CO bonuses are linear in n, so `E[C]` follows. For instance j of a shot, add the chance that the instances before j (which ones: `04` E-4.2) applied a new type; `08` E-8.10 owns this expectation, including n_COL,j and the cap. |
| Heat | one shared refreshing timer: Heat is present at time τ iff a Heat proc landed in the last `D_HEA` seconds (presence as in the row above); while present, the pool sums every Heat proc since ignition. The armor strip follows the step counter k_h, which outlives the episode (`03` E-3.8, `08` E-8.6). The pool has no steady state while Heat keeps being refreshed (`F:status` §4.4), so the Heat DoT contribution to `dps_burst` and `dps_sustained` is its expected mean over [0, `t_win`]. |

**Ramp-up.** Two time modes:

- *Steady* (`dps_burst`, `dps_sustained`, steady-state stack outputs): μ_s is the long-run mean seen by each
  consumer class at its arrival, not the time average λ_s·D_s: for a shot of a constant-rate schedule
  `μ_s = η_s·#{k ≥ 1 : k·T_per < D_s}` (η_s the expected random procs of s per shot, T_per the shot period); for a DoT
  tick the same count at its offset; equivalently, the per-period average of the window walk started from a converged
  state (`08`). The time average overstates the steady metrics by up to one event's procs, about 1/(D·r) relative
  for r events per second (+4.72 % at 1 shot/s with D = 6 against the `08` E-8.3 count at the arrival, `08` TV-08-17;
  the former gap G-13, now retired). Combo and conditional stacks at their
  steady values; the sniper combo, whose counter grows without bound, uses the pellet-weighted window mean of `06`
  E-6.16a (UNVERIFIED metric choice; T-12 replaces it). `dps_burst`: beam ramp full, spool full. `dps_sustained`:
  per-magazine spool-up and the steady-magazine beam-ramp mean η_rmp (section 4.1; `06` OQ-620).
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
pools. If the target is not dead at `t_ttk_max`, extrapolate phase by phase (OG, then SH, then HP) from the
remaining expected pools with the per-layer sustained rates (`dps_sustained` of the `overguard`, `shield` and `armor`
or `health` views; bypass types drain health while the shield is up; `08` E-8.21) and flag the value as
extrapolated. EV TTK is an expectation-path estimate, not the mean of the TTK distribution; the MC evaluator gives the
distribution.

### 4.3 Evaluator (b): event-driven Monte Carlo (MC)

Purpose: game-faithful simulation, distributions, validation of the EV evaluator, TTK.

- **Events.** Shot (from the schedule), pellet hit, radial hit, extra hit, proc application, stack expiry, DoT tick
  (per DoT instance; one shared Heat tick per second), Blast fuse expiry and detonation, Heat strip step, reload
  start and end, combo gain and expiry, Galvanized and arcane stack gain and expiry (`simulated` conditions), sniper
  combo decay, beam ramp, Magnetic break proc. Event times are held on the integer microsecond grid of CD-14 and
  compared exactly; events with equal grid time are processed in the order of CD-04 (the phases of `03` §7.1), then
  by event creation order (`08` §4.3).
- **RNG.** Counter-based generator (Philox4x32-10 or ChaCha8; `08` fixes one), keyed by
  `(seed, run, attack, event index, mechanic id)`, so results do not depend on thread count and two builds evaluated
  with the same seed see the same draws per mechanic (common random numbers for the optimizer's validation stage).
  Exact fractions are used; no rounding of chances to whole percent (D-017).
- **Runs.** Default `n_runs = 1000`, `seed = 0`; each run starts from the fresh state. Pass V simulates the window
  with a non-depleting target (layer views); pass L, with the same draws, simulates the `layered` target with
  depletion, shield gate and overflow (CD-08) until kill or `t_ttk_max`; pass L0 (parity outputs only, A0 > 0) is L
  with armor ignored, the inspiration's Health-panel TTK (`08` §6.4).
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
approximation (shape of the stack-count distribution, `08` E-8.3; independence; Heat window mean); `08` lists the expected gap classes. The optimizer
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
| D-005 | Incarnon flat split (01) | spread proportionally to type split incl. progenitor, ratio from listed total without progenitor (`addDmgToBase`, JS L13193-13205) | split not documented (`F:core` §3.1, open question 1) | X split proportionally to `Bw_t + V_t` (Bw: base after the Strength factor and base conversions, `01` E-1.3), summing exactly to X; UNVERIFIED (OQ-030) | — |
| D-006 | Violet shard Electricity position (01) | combined in slot order (shard slots 23-27, before innate elements) (`I:stat` §4.1) | added after modded elements, ordered with innate and progenitor elements by HCET (`W:Violet_Archon_Shard`, `F:buffs` §5) | game order; UNVERIFIED for other non-mod elements (OQ-024) | — |
| D-007 | Phoenix Talons ("Madurai") (01, 04) | ×1.3 on physical types, not on exalted weapons (JS L9254-9256, L9495) | +0.30 × total base (elements included) added as physical, split by physical shares, after physical mods; applies to exalted weapons (`W:Focus/Madurai`, `F:buffs` §6) | game formula; base-bucket scaling UNVERIFIED (OQ-020) | User (T-19) + Compat |
| D-008 | Faction mods vs Murmur, Corpus Amalgam (01) | no Murmur key → NaN → ×1 (Q15); Corpus mods do not hit Corpus Amalgam (`I:model` Q2) | Murmur bane works; Corpus/Grineer/Infested mods apply to Amalgam/Kuva/Deimos (`W:Faction_Damage_Bonus` §Mechanics) | game | — |
| D-009 | Impact-to-Puncture swap (01) | overwrites an existing Puncture value (`swapDmgType`, JS L7342-7350) | no in-scope PvE conversion mod exists (the "converted into Impact/Puncture/Slash" mods are Conclave-only, `W:Comet_Rounds` L3, `Module:Mods/data` `Conclave = true`; the beast-claw conversion mods belong to companion AI weapons, out of scope); the beast-claw wording "convert all ... to Impact" is weak evidence for adding (`01` §4.4) | add (generic `convert_base` hook); UNVERIFIED | — |
| D-010 | Riven Weak Point Critical Chance (02, 04) | stored under `SMITE`, no effect (JS L14722-14725; `I:stat` Q14) | relative crit chance on weak-point hits (`F:addmul` §11) | game | — |
| D-011 | Riven Weak Point Damage (02, 04) | mapped to the headshot-multiplier bucket `mult_for_head` (JS L14728) | weak-point damage bonus ΣWP (`F:addmul` §10-§11) | game | — |
| D-012 | Custom "Weak Point Increase Damage (as Primary Acuity)" (02) | adds the raw value to the head part `3 + x` (JS L9070-9076) while the Acuity mod itself is stored as 5.25 = 1.5 × 3.5 | ΣWP counts 1.5× on head weak points (`W:Enemy_Body_Parts`) | the custom stat enters ΣWP (head part gains 1.5 × value) | — |
| D-013 | Head-crit (02) | every headshot crit uses `2·CDM` (JS L8319-8328) | only on head-crit parts; no head-crit on Corpus humanoids, most Murmur and Techrot, MOA packs; 1×-headshot weapons never (`W:Critical_Hit`, `F:core` §10) | game; per-enemy part data (`05`) | — |
| D-014 | Absolute crit damage placement (02) | `crit_mult_add` (Crepuscular, Tenacious Bond) and Melee Doughty added after tier scaling: `1 + k(CDM − 1) + abs` (JS L8314-8328; `I:stat` Q10) | added to CDM before tiers: `1 + k(CDM − 1 + abs)` (`W:Arcane_Crepuscular`, `F:core` §9) | game | Compat |
| D-015 | Cold crit damage (02, 03) | +0.05 per stack (+0.5 more at 10) added to the final multiplier after tier and headshot; also on AoE (JS L10821-10825) | +0.10 at 1, +0.05 per stack to +0.50 at 9, Frozen +1.0; absolute, inside CDM before tiers; not on AoE; 4 stacks max on bosses and overguarded units (`W:Damage/Cold_Damage`, `F:status` §4.5) | game | Compat |
| D-016 | Hunter Munitions (02) | needs a crit, then rolls `0.3 × min(CC, 1)` (JS L10761-10766): P = 0.3·CC² below 100 % CC | 30 % per critical damage instance (`W:Hunter_Munitions` lines 2, 38) | game; tier independence UNVERIFIED (OQ-018) | Compat |
| D-017 | Fractional chances (02) | crit, status and multishot fractions rounded to whole percent before rolling (JS L10005, L10012, L10054, L10679) | exact | exact | Compat |
| D-018 | Crit tier on a failed roll (02) | tier from `CC − 99`: one tier too many when frac(CC) ∈ (0.99, 1) (JS L9826) | `floor(CC)` (`F:core` §9) | E-6 | — |
| D-019 | Multishot floor (02) | none for negative riven multishot (JS L8983-9006) | "When affected by a Multishot curse, Multishot can no longer go below 1 so it will only have an effect if the weapon has innate Multishot ..." (`W:Multishot` L197) | `MS ≥ 1` for every weapon, innate-multishot weapons included | — |
| D-020 | Proc type selection (02) | 100-slot mask with `ceil(share)` slots per type; last types lose up to n − 1 points (JS L12239-12246, L10022) | exact proportional weights (`W:Status_Effect` §Damage Distribution) | exact | Compat |
| D-021 | Encumber random status (02, 04) | `round(random × 12)`: Impact and Viral at half weight (JS L10036, L12875) | "Random proc is uniformly chosen from the following and is independent from the weapon's damage distribution" (13 listed statuses); at most one added status per instant of time, shared by simultaneous hits including explosions (`W:Secondary_Encumber` L31, L34, L41-54) | uniform over the 13 listed statuses; one Encumber proc per `08` proc event (the modelling of "instant" is UNVERIFIED, `02` OQ-231) | — |
| D-022 | Stealth damage (04, 07) | melee non-crit ×7; crit `c + 7`; weapon rank ignored (`getStealthDmgBonus`, JS L8330-8336) | `× (1 + SDB)` non-crit = ×8 at rank 30; crit `1 + SDB + k(CDM − 1)`; `SDB = 1 + 0.2·rank`, capped at rank 30 (`W:Stealth` lines 36-49); weapons with maximum rank 40 `1 + 0.15·rank` (`W:Finisher` L103-104) | game (E-12) | User (T-08) + Compat |
| D-023 | Weak-point crit chance multiplicative (02) | hidden option `o.w` and per-attack `isWPCritMult` multiply CC by `(1 + WP CC)` (JS L8201-8210) | additive in the relative bucket (`W:Primary_Acuity`, `F:addmul` §10) | additive; option kept for link import | User (T-21, hidden) |
| D-024 | Proc weights quantized (02) | weights from unquantized modded damage (JS L10063-10077) | not stated (`F:status` §8 open 2) | weights from `q_t`; UNVERIFIED (OQ-004) | — |
| D-025 | DoT "+1" accumulator (03) | none (JS L9154-9204) | `tick = (Σ seeds + 1)·C·M`, once per merged Heat/Electricity/Gas pulse, per stack for Slash/Toxin; the crit multiplier is inside the seeds ("critical seeds of 80.0195389"; "(80.0195389 + 1) × 0.35 = 28.3568382"), so the "+1" is not scaled by c_k (`W:Damage/Calculation` §DoT, cache L619-635, single editor 2026) | apply, with c_k and Part_dot inside the seed (E-13); UNVERIFIED (OQ-023; Part_dot placement by analogy, `03` OQ-301) | Compat |
| D-026 | Heat armor strip (03, 05) | flat 50 % while any Heat stack is live (JS L10815) | ramp: ×0.5^(1/4) every `0.5·(1 + SD)` s to 50 % at 2 s; returns in 4 steps every 1.5 s over 6 s after Heat ends (`W:Damage/Heat_Damage` §Armor Stripping, edited 2026-11-14) | game | Compat |
| D-027 | Heat stacking (03) | each Heat proc expires on its own timer in the DPS numbers; TTK uses a shared refreshing timer (JS L11139-11143, L13690-13703) | one shared timer refreshed by every new proc; all procs pool into one tick per second; duration = longest SD among stacks (`W:Damage/Heat_Damage` §Stack Combining) | game, both evaluators | Compat |
| D-028 | Heat inherit (03) | custom stat `heatInherit` replaces the Heat elemental bonus (JS L9154-9204) | a burning target keeps the first source's Heat elemental and faction bonus for later procs; inherited faction applies once (`W:Damage/Heat_Damage` §Heat Inherit) | external first source via the T-26 custom stats `heatInherit` and `heatInheritFaction` (F_inh); same-weapon later procs keep faction² (UNVERIFIED, OQ-006) | User (T-26) + Compat (`03` §10.3: F_tick,p = F_i, faction squared, and `E^dot_HEA` = the custom value or the procs' own) |
| D-029 | Viral and Magnetic after expiry (03) | multiplier keeps its last value after the last stack expires (`I:status` Q6, JS L10320-10321, L10779-10804) | multiplier is 1 without stacks (`W:Damage/Viral_Damage`) | game | Compat |
| D-030 | Own-hit and same-shot timing (03) | a hit sees its own procs and earlier pellets' procs for every debuff (`timelineStatus`, JS L10093-10136) | a hit deals damage before its procs; sibling pellets do not share Viral; CO counts earlier pellets (`W:Damage/Viral_Damage`, `F:addmul` §1.2) | CD-04 | Compat |
| D-031 | Target-side terms on DoT ticks (03) | Viral, Magnetic and armor strip frozen at the inducing hit (`I:enemy` §3.3, §9.2) | evaluated when each tick lands (`W:Damage/Viral_Damage`) | CD-05 | Compat |
| D-032 | Puncture duration (03) | 6 s (JS L5547) | 10 s (`W:Damage/Puncture_Damage`, `W:Status_Effect` tables; `W:Damage` table still says 6 s, outdated) | 10 s | Compat |
| D-033 | Impact duration (03) | 1 s (JS L5545) | 6 s per stack (`W:Damage/Impact_Damage`; `W:Status_Effect` "1 second" is the flinch) | 6 s; UNVERIFIED (conflicting tables) | — |
| D-034 | Blast (03) | unlimited stacks, no detonation; damage `0.3·` scaled by `parseInt(1.5·(1 + SD))` and Blast element mods; ignored by TTK (JS L5571, L9177, L11356-11360, L13669) | 1.5 s fuse per stack (SD lengthens it), one 0.3·H hit at expiry; 10 stacks (4 bosses) or death detonate all; no elemental scaling (`W:Damage/Blast_Damage`, edited 2026-11-03; `W:Damage_over_Time` disagrees on scaling; `W:Damage/Calculation`, edited 2026-11-04, disagrees on host damage at detonation, `03` D-306) | Blast page; elemental scaling UNVERIFIED (OQ-007); host damage at detonation UNVERIFIED (`03` D-306, OQ-306) | Compat |
| D-035 | Gas (03) | one Gas proc per hit credited `gasStacks × tick`; drops non-crit headshot factor; extra faction in non-crit path (JS L11219-11290; `I:status` Q22) | each proc: 6 ticks at t + 0 .. t + 5, own timer, cap 10 (`W:Damage/Gas_Damage`) | game | — |
| D-036 | Status duration rounding (03) | `parseInt(D0·(1 + SD))` for every status, incl. debuffs, Blast and Toxic Lash (JS L9085-9093) | DoT ticks rounded down (Lasting Sting note); debuff rounding unknown | CD-06; UNVERIFIED (OQ-005) | Compat |
| D-037 | Toxic Lash DoT duration and Saryn's passive (03, 04) | fixed 7.5 s for Toxic Lash procs only (7 ticks credited in DPS, 8 in TTK); the weapon's own procs get no Saryn bonus (JS L5555-5561, L13706-13713; no Saryn code in the bundle) | Toxin DoT `floor(6·(1 + SD))` ticks; Saryn passive +25 % SD on every status of Saryn's weapons (`W:Toxic_Lash`, `F:buffs` §1.4, `W:Saryn/Abilities/Passive`) | Saryn's passive (+0.25 SD; "Status Effects inflicted by Saryn's weapons and abilities last 25% longer", additive with Lasting Sting, `W:Saryn/Abilities/Passive` L1-2) applies to every proc of the build when Saryn is the frame; T-16 implies Saryn is the frame (Toxic Lash is not subsumable); counted once if also selected in `04` | — |
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
| D-052 | Enemy-specific resistance and armor maps (05) | (1) `elemRes` ignored on armor-layer direct hits; (2) the Armor-row TTK (`armorFull` pass) uses the Toxin-only map on shielded targets with Toxin attacks (`JS L11365`; `I:queue` QM16-QM17; the former `08` D-806). The Toxin-only health and armor panel maps are `05` D-504 (Compat) | resistances apply to every instance; every type reaches health and armor once the shield is gone | consistent application | — for both halves: shielded Toxin scenarios are not oracle-safe for the Armor-row TTK |
| D-053 | Status cap of capped enemies (03, 05) | `maxProcStacks` limits same-type procs inside one hit and live stacks, never DoT damage (JS L10011-10031, L10119-10127) | status-capped enemies (Kuva Liches "no status effect will exceed 4 stacks", U27.3, `W:Bosses` L925; Acolytes; Archons in Archon Hunts) 4 for every status (Kuva Liches Impact 5, Acolytes Impact 3); other bosses Cold 4 and Blast 4 only (`W:Damage/Cold_Damage` L27, L364; `W:Damage/Calculation` L643-645); per-type caps on current pages | CD-07; UNVERIFIED for DoTs (OQ-008) and for the other statuses of ordinary bosses (`05` OQ-531) | Compat |
| D-054 | Magnetic break proc (03) | not modelled (`I:status` §5.2) | on shield or overguard break: forced Electricity proc, `0.03·n_MAG·max pool` over 6 s, status damage twice, faction twice (`W:Damage/Magnetic_Damage`, U36) | model (MC; EV in TTK only) | — |
| D-055 | Steel Path flat base-health bonuses (05) | not modelled (`I:enemy` §2.6) | Juggernaut +1,800 etc. (`Module:Enemies/data` `SteelPathHealthBonus`) | add to base before ×2.5 (gadget order; UNVERIFIED) | — |
| D-056 | CO status count (04) | number of damage types in the attack's modded damage (+1 for external Viral), static (`getCountDifferentStatuses`, JS L12015-12020) | distinct status types on the target from any source, excluding the hit's own (`W:Condition_Overload_(Mechanic)` §What Counts) | expected or simulated count | User (T-05) |
| D-057 | CO additive vs multiplicative (04) | global option or per-attack data flag `isCoMult` (JS L9235, L9393) | per-attack catalogue: behaviour, ratio `B_co/B_ref`, "does not apply" list (`W:Condition_Overload_(Mechanic)`) | catalogue (E-11); default `adding`, ρ_co = 1 | User (T-04) |
| D-058 | Adding-CO omissions (04) | CO term multiplied by Eclipse, `multiple` and the other map multipliers (JS L9389-9401, L9492-9498) | Eclipse, Garuda, Parasitic Link, Overdriver, Duality, Furious Javelin, Hall of Mirrors, Longbow Sharpshot, projectile Primary Compression, Dragon Key, falloff, bow charge omit the CO term (`F:addmul` §1.6) | E-11 with `G_F` | Compat |
| D-059 | CO on AoE and Incarnon flat (04) | off for every `shot_type == "AoE"` and slam; the Incarnon-flat path applies additive CO even with conditions off, on AoE and on slams, and drops Sharpshot (JS L9402-9414; `I:stat` Q2) | hitscan explosions give CO to the directly hit target from the hitscan damage; flat excluded from B_co on Adding attacks (`F:addmul` §1.3, §1.7) | catalogue rules | — |
| D-060 | Secondary Shiver (04) | +4.5 whenever conditions are on, Cold stacks not read (JS L9365) | +0.45 per Cold stack on the target (`W:Secondary_Shiver`) | per stack (expected or simulated) | Compat (n_COL = 10 whenever conditions are on; `04` OQ-434) |
| D-061 | Conditional stacks over time (04) | Galvanized, arcanes, on-kill mods fixed at max stacks under "trigger all conditions", else ignored (`I:queue` §8) | stacks ramp and decay with their triggers (`F:mods` §6.1) | T-02 modes `all_max`, `none`, `simulated` | User (T-02) |
| D-062 | Light-attack combo (04, 07) | combo-scaled bonuses use 12× when the option is off (`combo_counter = 11`, JS L8219-8224, L9906-9910) | combo ramps from Initial Combo with hits (`W:Melee_Combo`) | option off → simulated ramp | User (T-10) + Compat |
| D-063 | Combo range (07) | combo inputs accept 1-13 for every weapon (`I:feat` quirk 5) | cap 12×; Venka Prime 13×; Dex Nikana 11 per tier, cap 11× (`W:Melee_Combo`) | clamp to the weapon's cap | — |
| D-064 | Heavy-attack combo model (07) | combo spent every `int(0.8·hits/T) + heavyHits` hits, never rebuilt, Initial Combo restored instantly (JS L12158-12181) | combo gained per stance hit, spent by heavies with HAE retention (≤ 0.9), Initial Combo regenerates at 40/s (`W:Melee_Combo`, `W:Melee`) | game | Compat (`07` E-7.39) |
| D-065 | Melee hit timing (07) | hits spread evenly over the window; heavy count from the default stance table even with a stance mod; slams 1/s and forced `statuses[0]` on every slam hit (JS L10137-10216, L12115-12150) | stance combo durations and per-hit lists; per-weapon heavy, wind-up and slam attacks (`Module:Stances/data`, `F:melee` §1-§4) | game data; uniform spacing inside a combo (UNVERIFIED) | Compat (`07` E-7.38) |
| D-066 | Sniper combo (06) | fixed multiplier, 1.5 by default even with the option off (JS L10326-10330) | count rises per hit, `M = 1.5 + 0.5·floor(log3(count/MinCombo))`, decays (`W:Sniper_Rifle`) | simulated unless T-12 fixes it | User (T-12) + Compat |
| D-067 | Spiteful Defilement (04) | doubles the whole crit multiplier when total stacks < 3 (JS L10752-10760) | "Despite what the in-game description says, the bonus is a +1x flat bonus to critical damage, calculated after percentage modifiers, similarly to Tenacious Bond's bonus" (`W:Phenmor` L133-139); applies with fewer than 3 distinct statuses | +1.0 in ΣCD_abs when `n < 3` (with Vital Sense and CDM0 2.0: CDM 5.401, where ΣCD_rel would give 6.402; `02` TV-02-01) | — |
| D-068 | Melee Careen and Archon Electricity ability shard (04) | unconditional multipliers (JS L10466, L10389); the shard applies to exalted weapons | Careen ×2.5 only vs Frozen targets; ability-damage shards never apply to exalted weapons (`W:Melee_Careen`, `W:Archon_Shard`) | game | — |
| D-069 | Per-status status-damage bonuses (03) | Emerald Toxin etc. added to the product `zb` (outside `multiple` and other factors) (JS L11003, L11084) | additive with Elementalist inside one bucket (`W:Archon_Shard` line 337) | inside A_sd (E-13) | — |
| D-070 | Hunter Munitions proc as a stack (03) | its Slash DoT is credited but never enters the timeline (JS L10761-10766) | it is a Slash status | counts for stacks and CO | — |
| D-071 | Xata's Whisper (04) | double-counts headshot and vulnerability, ×21 in non-crit branches, innate DR inside the armor argument (JS L11403, L11584, L11763, L13226-13243) | E-14 (`W:Extra_Hit`, `W:Xata's_Whisper`) | E-14; Blast-detonation trigger modelled in MC (UNVERIFIED magnitude) | — |
| D-072 | Toxic Lash damage types (04) | the extra hit inherits the weapon's per-type modifiers (JS L11465-11495) | it is a Toxin instance (`W:Toxic_Lash`) | E-14 | — |
| D-073 | Primary Debilitate faction (03) | faction applied a third time on its procs (JS L11459) | "The extra Status Effect created by this arcane is applied as a separate damage instance, causing Faction Damage Bonuses to multiply the Damage over Time effect of Heat, Electricity, and Toxin status three separate times" (`W:Primary_Debilitate` L31; `W:Extra_Hit` §Weapon Effects, `04` §6.5) | three times: the inspiration's count agrees with the game; the seed of the 0-damage extra hit's proc is UNVERIFIED (`03`, `03` OQ-313, `04` OQ-414) | — (seed rule not compared) |
| D-074 | Nightwatch Napalm (03, 06) | DoT base 75 overwritten by 150; progenitor scaling; credited at hit (JS L10409-10441) | blaze 150 Heat per tick (the page says the description is outdated), 6 ticks from 0 s, once per second, no crit; Napalm Heat procs last D0 = 3 s (`W:Nightwatch_Napalm` §Notes) | page values: the inspiration's 150 agrees; the differences are progenitor scaling and crediting at hit; UNVERIFIED | — |
| D-075 | Bow cycle (06) | `1/(CT + reload)` per shot plus a magazine reload after every shot: interval `CT + 2·reload` (JS L12290-12293, L12333-12348) | `EFR = 1/(CT + Reload)`, no separate reload (`W:Fire_Rate`, `W:Bow`) | game | Compat |
| D-076 | Charge weapons (06) | `1/(CT + reload)` per shot plus magazine reloads; only shot 0 pays the charge (JS L12296, L12351) | `EFR = 1/(CT + 1/FR)`; Lanka `1/CT`; magazine reload as usual (`W:Fire_Rate`) | game | Compat |
| D-077 | Semi-auto cap (06) | none; no `trigger` field read (`I:agg` §2.3) | 10 shots/s (Komorex ≈ 8) (`W:Fire_Rate` §Mechanics) | cap 10 with per-weapon override; UNVERIFIED value (OQ-012) | — |
| D-078 | Beam damage ramp (06) | not modelled (`I:agg` §2.3) | 20 % → 100 % over 0.6 s, decays after 0.8 s idle over 2 s; per-weapon starts (`W:Continuous_Weapon`) | linear ramp; UNVERIFIED shape and starts (OQ-013) | Compat |
| D-079 | Spool-up (06) | no spool code in the scheduler (JS L12251-12406) | rate rises from a start fraction per shot (Gorgon 9 shots from 20 %) (weapon pages, `F:weapon` §6) | game; reset rule UNVERIFIED | Compat |
| D-080 | Burst delay (06) | divided by `(1 + speed)` and multiplied by `(1 + speed_mult)` (JS L8936-8952) | `BD / (1 + max(0, ΣFR))` (`W:Fire_Rate` §Burst; inspiration changelog 2026-01-28 agrees on scaling) | game; UNVERIFIED numerically (OQ-014) | — |
| D-081 | Shot-count bound (06) | planned count `Pa` truncates: rates below `1/T` give 0 shots (JS L12299-12321) | — | event loop only | — |
| D-082 | Battery and per-shell reload (06) | battery recharge uses the ammo-efficiency-inflated magazine; `reloadRate` mods divide a rate; no 0.5 s delay floor; per-shell reloaders use the flat `base/(1 + reload mods)`, ignoring the magazine dependence (JS L9019-9041, L12278-12287; `I:stat` Q21) | battery: `δ/(1 + RS) + Mag/Rate`, and reload-speed bonuses cannot push δ below 0.5 s (data delays below 0.5 s exist, Dex Pixia 0.25; `W:Reload` L54, `06` OQ-608); per shell: `(t_se + t_sh·Mag)/(1 + RS)` (`W:Reload`, `W:Battery_Weapon`) | game | — |
| D-083 | Ammo cost and efficiency (06) | 0.5 ammo for every attack of a BEAM-tagged weapon; efficiency up to 0.999999 (JS L12267-12273, L12464-12471) | per-attack `AmmoCost`; efficiency capped at 0.75 (Hotfix 44.1.3); infinite-ammo states separate (`W:Ammo`) | game | — |
| D-084 | First- and last-shot marks (06) | groups of `perMag` shots by index, also for infinite magazines (JS L12182-12196) | first shot = counter at Mag − 1 after a shot; last shot of a real magazine (`W:Primed_Chamber`, `W:Synth_Charge`) | game | — |
| D-085 | Damage falloff (06) | display only (JS L7841-7842) | applied by distance; progenitor part falls 100 % → 0 % linearly; Incarnon flat ignores falloff (bug) (`W:Damage_Falloff`) | applied at distance T-27 (default 0 m) | User (T-27) |
| D-086 | DoT accounting in DPS (08) | full DoT credited at the applying hit, never truncated at the window (JS L10982-10986) | ticks land over time | landed for our metrics; credited for parity outputs | Compat |
| D-087 | DPS statistic (08) | median of 1-s buckets filled by hit index, not time (JS L12787-12805) | — | `dps_window` (mean landed damage / T); parity output kept | Compat |
| D-088 | Averaging (08) | runs for 5 s wall-clock; "Avg" shows a median (JS L14273-14485) | — | fixed `n_runs`; mean and median labelled | — |
| D-089 | TTK simulation (08) | base durations (SD ignored), Blast ignored, Toxic Lash 8 ticks, per attack only, frozen target-side terms (JS L13593-13804) | game rules of `03` and `05` | one event model for DPS and TTK | Compat |
| D-090 | Combine attack effects (08) | timelines merged by hit index; merged procs never expire (JS L12540-12549) | — | merge by time, normal expiry | — |
| D-091 | Index-mode status extension (08) | uses `round(N/T)` hits per second; 0 below 0.5 hits/s (JS L10295-10310, L10349) | — | time-based only | — |
| D-092 | Mod capacity and polarity (09) | not modelled (`I:opt` §8) | capacity, drain, polarity, Forma rules (`W:Mod`, `W:Polarity`, `F:mods` §2) | optional constraint modes (09); capacity mode `any_forma` by default | User (T-35, T-36) + Compat (capacity mode `ignore`) |
| D-093 | Mod incompatibility (09) | substring name test plus asymmetric `uncomp` lists (JS L6998-7007, `I:model` Q18-Q19) | same hidden parent type; wiki `Incompatible` lists, union of both directions (`W:Mod/Compatibility`, `F:mods` §5) | wiki lists | — |
| D-094 | Custom critical chance on heavy attacks (07) | custom crit chance added once per non-empty slot and twice in the per-hit path (`I:stat` Q7-Q8) | — | added once | — |
| D-095 | Sonar on DoT (03; wiki-versus-wiki, section 1.4 rule 3) | no Sonar input (no Sonar code in `min.pretty.js`) | `W:Sonar` L25-26: DoTs "will have their damage calculated on the resulting damage if proc'd from a hit that hits a Sonar weakspot" (revision 2813083, 2026-09-25, after the U44 rework: "All status DoTs are affected by Sonar now"); loser: `W:Damage_over_Time` L171 and `W:Damage/Slash_Damage` L60 ("only increase damage of the initial hit, not the resulting proc damage"; pre-rework text, the DoT page's only later edit, revision 2817738 of 2026-10-12, changed one Thermal Transfer line) | `Part_dot = Part` (E-13); Detect Vulnerability by analogy, UNVERIFIED (`03` OQ-312) | — |

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
| T-03 | Hit location ("Headshot/Weak Point bonus active", `o.hs`) | `body` (default), `head`, `weak_point` (non-head, ours), `part(<name>)` (ours: any part of the enemy's `05` part list by name, such as Juggernaut maw or belly or a crit-promotion part; `02` OQ-229, `05` OQ-523) | every direct damage instance (not radial, not extra hits except Xata, DoT inherits) uses E-7 with the selected part from `05` (`head` = first part with the headshot flag; `weak_point` = first non-head weak point, else body); `torso_only` attacks (Peacemaker, `02` §4.10) ignore T-03 and hit the body part; radial instances use the body part (the parent instance's part for `radial_parent_part` attacks, `02` §4.10); weak-point-conditional effects trigger |
| T-04 | CO multiplicative ("damage per status calculated as multiplicative", `o.md`) | off (default) / on | on: every attack whose CO behaviour is `adding` uses `multiplying` with ρ_co = 1; `none` stays `none` |
| T-05 | CO status count ("Manual selection of the total number of statuses", `o.ms`; plus ours) | `expected` (default), `weapon_types` (inspiration rule), `manual(N)`, N ∈ 1..50 | n for CO-type bonuses: `expected` = section 4.2 (EV) or the live target state (MC); `weapon_types` = number of nonzero damage types of the attack after combination, +1 if T-06 is on and the attack has no Viral; `manual` = N |
| T-06 | External Viral (`o.v`) | off / N ∈ 1..10, default off (10 when switched on) | the target permanently holds N Viral stacks from another source; own Viral stacks add, capped at the Viral cap (enemy cap applies); Viral counts as present for n. T-06 is T-43 restricted to Viral, kept for link import |
| T-07 | External armor strip (`o.d`) | 0-0.99, 0 | an ability-type strip of total armor in E-15 (additive with Corrosive Projection) |
| T-08 | Stealth (melee) (`o.s`) | off / on; weapon rank 0..maximum rank (30 or 40), default the maximum | every melee hit uses `c_k = 1 + SDB + k(h·CDM − 1)` with SDB of section 2.4 (`1 + 0.2·min(rank, 30)`, or `1 + 0.15·rank` for maximum-rank-40 weapons); the DoT of such a hit inherits it |
| T-09 | Heavy attack (melee) and start combo (`o.h`) | off / on; start multiplier 1..weapon cap, default 12 | attacks without an explicit heavy flag are evaluated as heavy attacks (`07`); the first heavy uses the start multiplier; consumption and regeneration per `07` |
| T-10 | Light-attack combo multiplier | off (simulated ramp) / M ∈ 1..cap, default off | M fixes `combo_mult` for combo-scaled bonuses (Blood Rush, Weeping Wounds, Gladiator); light attacks are never multiplied by the combo multiplier |
| T-11 | Combo ability multiplier (exalted, `o.ac`) | 1..12, off | multiplies exalted pseudo-weapon damage by the ability combo value (`07`) |
| T-12 | Sniper combo multiplier (`o.sc`) | off (simulated count) / M ∈ 1.5..100 | M fixes `sniper_combo_mult` for sniper-combo attacks |
| T-13 | Roar (`o.r`) | off / value, default 0.30 when on | `sum_faction += value` (value is final, already Strength-scaled) |
| T-14 | Eclipse (`o.z`) | off / value, default 0.30 | factor `1 + value` in `G_F` |
| T-15 | Xata's Whisper (`o.x`) | off / value, default 0.26 | extra hit E-14 with `p_xh = value` |
| T-16 | Toxic Lash (`o.l`) | off / value, default 0.30 | extra hit E-14, `p_xh = value` (×2 for melee), forced Toxin proc; implies Saryn is the frame, so Saryn's passive adds +0.25 SD to every proc of the build (D-037) |
| T-17 | Nourish (`o.n`) | off / value, default 0.45 | `sum_elem[VIR] += value`, non-combining |
| T-18 | Flat crit chance ("~Kullervo, Harrow", `o.j`) | off / value, default 0.50; mode `flat` (default) or `covenant` (ours) | `flat`: `sum_cc_abs += value`; `covenant`: the value is Covenant's v_cov, entering ΣCC_abs as `min(v_cov, 0.50)` on non-weak-point hits and `min(4·v_cov, 2.00)` on weak-point hits (`02` E-2.4; `04` OQ-421) |
| T-19 | Phoenix Talons ("Madurai", `o.g`) | off / on | Phoenix Talons per D-007 |
| T-20 | Ability strength (`o.t`) | 0-10, 1.0 | `strength` (Str) for exalted weapons and Strength-scaled weapon effects |
| T-21 | Weak-point crit chance multiplicative (`o.w`, hidden) | off / on | on: weak-point crit chance multiplies `(1 + WP CC)` instead of adding (D-023); kept for link import |
| T-22 | Combine attack effects (popup) | per family (section 2.3); default on in the game profile, off in the compat profile | on: the family is one evaluation unit whose attacks share one target status timeline merged by time, with group TTK (CD-12, `08` E-8.23); off: every attack is its own unit (parity); two firing attacks of one weapon grouped by the user keep independent schedules (`06` OQ-628) |
| T-23 | Enemy options (`05`): level; Steel Path; Eximus and Eximus kind (`generic` default, Entropic, Guardian); party size 1..4 (1..6 in The Icebind), which feeds squad scaling and the mission modifiers; mission modifier (`05` §7.5); `armor_hydroid`; squad Corrosive Projection count and Coaction Drift; external shield strip (`05` E-5.16); flat base-armor strip rate; target inside a Guardian or Prosecutor aura; `context_overguard`; `shield_removed_externally` and `pt_shield_type` (`05` E-5.26) | `05` defaults (none or `generic`); party size 1 (solo, Decision) | `05` (E-5.4, E-5.5, §4.1 Eximus kind, §7.5, §7.6 and E-5.27 auras, §9.3-§9.4, E-5.11 flat strip rate, §10.3, E-5.26) |
| T-24 | Progenitor element and value (`j`, `jv`) | 7 elements; 0.25-0.60, default 0.60 | `progenitor_type`, `progenitor_frac` |
| T-25 | Kitgun grip and loader, zaw grip and link, Chromatic Blade element, Reactive Storm equipped | per item | inputs to ST-1 (`06`, `07`, `04`); Reactive Storm's element is derived from the target's `damage_faction` (`01` Table 4.10-A; "consistent regardless of Mods", `W:Reactive_Storm` L56); a user-chosen type exists only as a compat link-import option (`01` D-101) |
| T-26 | Custom stat values (popup, `p.*`) | see 6.3 | each value is added to the bucket in 6.3 |
| T-27 | Target distance (ours) | 0-1000 m, 0 | `distance` d for falloff (`06`) |
| T-28 | Unique damage reduction (ours) | on (default) / off | off: attenuation, M_dps, adaptation and every flat enemy DR (innate, class, status and part DR) are ignored (`05` §15.5). Off is not inspiration parity: the inspiration always applies its own `innateDR` table (32 enemies); oracle tests avoid the enemies listed in `05` OQ-522 in both settings |
| T-29 | Monte Carlo runs and seed (replaces "Calculate average values over several cycles") | `n_runs` 1-100,000 (1000), seed (0) | section 4.3 |
| T-30 | Melee plan (`07` §8.1) | plan per selected melee attack (`light_loop(K)`, `slide_loop`, `heavy_loop`, `build_spend(M_trig)`, `tennokai`, `slam_loop`, `heavy_slam_loop`, `glaive_cycle`, `cycle_loop`, `data_attack_loop`) with its parameters: combo K, M_trig, heavy policy (`full_regen` default or `back_to_back`), `heavy_cancel` (off) and t_dmg, T_slam (1.0 s), glaive throw kind (charged), T_cyc; defaults of `07` §8.1 | the melee schedule generator of `07` (OQ-722) |
| T-31 | Exalted ability rank (ours) | 0..3 per exalted weapon, default 3 | rank r of `01` E-1.1 (`R_r = s_r/s_3` for model 1, m_r and p_r for models 2 and 3; `W:Exalted_Weapon` §Notes) |
| T-32 | Weak spots (ours) | per enabled Sonar or Detect Vulnerability source σ: location `L_σ ∈ {hit_part, torso}` (default `hit_part`) and node count ν_σ (Sonar default 1; Detect Vulnerability always 1) | `spot_σ` and `M_son = Π_σ s_σ^(ν_σ·spot_σ)` of `02` E-2.10 (`02` OQ-225) |
| T-33 | Strength sources (ours) | `derive` (Strength from the build's `04` §5 sources, Parasitic Link multiplicative) or `manual` (T-20, default); per-buff caster Strength (default 1.0 for squad buffs); self-cast Ability Duration Dur_e (default 1.0) | `04` §4.4 (OQ-420) |
| T-34 | Finisher mode (ours) | off (default) / stealth / back / front / universal | `04` §7.2 (OQ-437) |
| T-35 | Capacity mode (ours) | `ignore`, `current`, `any_forma` (default), `budget(N)`; allowed Forma types; `frame_complete` for warframe capacity | `09` E-9.18 (OQ-915; `ignore` reproduces the inspiration, D-092) |
| T-36 | Lower mod ranks in the optimizer (ours) | off (default) / on | on: the optimizer may choose ranks below the maximum (`09`, OQ-915) |
| T-37 | Caster of T-13 Roar (ours) | `self` / `ally` (default) | the caster field of `04` §4.4 for T-13; feeds the Helminth rule `09` E-9.32 (OQ-915). Eclipse, Xata's Whisper and Toxic Lash act only on the caster's own weapons (`W:Eclipse` L9, `W:Xata's_Whisper` L8, `W:Toxic_Lash` L14), so T-14 … T-16 are always self-cast and E-9.32 always applies to them (adopted from `09` OQ-926) |
| T-38 | Aiming and zoom (ours) | aiming on (default) / off; zoom level, default the highest under `all_max` and none under `none` (`04` D-406) | `while_aiming` buffs, sniper scope rules and zoom-only children (`06` §5.9, §7.2; `06` OQ-619 (c), (d)) |
| T-39 | Incarnon switch time (ours) | off (default) / on | on: the form switch time T_sw (= reload time) is counted at form start and end (`06` §6.6; `06` OQ-619 (a)) |
| T-40 | Timed reload (ours) | off (default) / on | on: non-battery weapons skip the 1/FR reload lead (timed reload press, `W:Reload` L6; `06` §5.8, OQ-619 (e)) |
| T-41 | Pellet hit fraction (ours) | 0..1 per attack, default 1 | fraction of the instances of a shot (shotgun pellets, AoE children) that hit the target (`06` OQ-619 (b), `08` OQ-808); the value is a user input, no spread data exists |
| T-42 | Gauge uses per minute (ours) | ≥ 0, default 0 | class (c) usage rate of gauge-gated attacks: further uses scheduled without magazine cost after the first use from the full gauge (`06` §6.7, OQ-623) |
| T-43 | External statuses (ours; generalises T-06) | per status type s: off (default) / N_s stacks, 1..cap_s (COL 1..9) | the target holds N_s stacks of s from another source for the whole window, with no damage and no expiry. They count as present for n ("Bonus damage from status procs may originate from any source, including ... Warframe abilities, companions, other players, or the environment", `W:Galvanized_Shot` L48); they enter every stack-dependent function (Vir, Mag, c_corr, cold_cd, punc_cc, tau_sc, n_COL for Secondary Shiver and Shattering Frost; HEA: strip step k_h = 4 for the window, the episode's inherit source external per T-26); own stacks use the cap_s − N_s free slots (replacement acts among own stacks only, as T-06); external stacks fire none of the build's on-proc triggers. The per-status rules are Decisions (UNVERIFIED); `08`'s `n_ext` and `03`'s stack state read them |

### 6.2 Conditions toggle T-02

Every conditional effect in the effects catalogue (`04`) declares one condition class:

| Class | Examples | `all_max` | `none` | `simulated` |
|---|---|---|---|---|
| (a) target state | CO-type per status, Secondary Shiver, below-50 %-health perks, Melee Careen (Frozen), Spiteful Defilement | count-based bonuses use n per T-05; threshold conditions are true | CO-type and every other class-(a) bonus off | from the target state (expected in EV, live in MC; health thresholds only in the `layered` pass) |
| (b) own events | on hit, on crit, on headshot, on weak-point hit, on reload, aiming, first/last shot, combo, Galvanized Scope stacks | active at max stacks from t = 0 | inactive | triggered by the attack's own events, with stacks, durations and decay (`04`); timing-stat effects with random triggers: uptime input, see below |
| (c) external | on kill, on ability cast, while invisible, wall latch, holster, on damaged, Arcane Avenger | active at max stacks from t = 0 | inactive | user uptime and stack inputs per effect, default max |
| (u) user-enabled buff | ability buffs and Helminth values, target vulnerabilities and debuff abilities, squadmates' auras, focus, companion, squad and environment buffs, weapon-made buffing fields, T-13 … T-19 | user inputs (value or caster Strength, stacks m_e, uptime u_e; defaults max and 1) | user inputs (unchanged) | user inputs (unchanged) |

Class (u) holds every effect that exists only because the user put it in the scenario's buff list or switched on
T-13 … T-19: its activation is the user's statement, not a condition of the build, so T-02 does not switch it
(`04` §4.1, §4.3; the inspiration applies Roar, Eclipse, Xata's Whisper, Toxic Lash and Nourish whenever their
checkboxes are set, JS L9104-9109, while "Trigger all conditions" gates only `WITH_COND` values, JS L8668-8744).
Conditions inside a class (u) effect (Grave Spirit vs Doomed, Biting Frost vs Frozen, Firing Line vs Lifted) are
class (a) sub-conditions and follow T-02; gates (melee only, headshots, weapon class) apply as gates.

First/last-shot multipliers (Primed Chamber, Synth Charge) are class (b) but are always driven by the magazine
schedule, also in `all_max` (the inspiration does the same, `I:queue` §3.5).

Effects on timing stats (fire rate, attack speed, reload, charge time, burst delay, magazine, ammo efficiency) change
the schedule, which never depends on random draws (section 4.1), so they are never driven by random events: under
`all_max` and `none` they are on at max or off for the whole window; under `simulated`, deterministic schedule triggers
(on reload, on reload from empty, first and last shot) follow the schedule, and random or external triggers (on crit,
on headshot, on proc: Arcane Acceleration, Velocity, Tempo, Strike, Shotgun Vendetta and others) use the class (c)
uptime input u_e (default 1) in both evaluators: a numeric bonus enters as `u_e ×` its maximum value (`06` §5.9,
`04` §4.3). The fixed-point alternative (iterating the trigger rate with the fire rate) is recorded in `04` OQ-432.

### 6.3 Custom stats (T-26)

The inspiration's 26 custom stats (`I:feat` §9), plus one companion input of ours. Percent inputs are fractions in the
engine.

| Inspiration key (link) | Label | Engine bucket |
|---|---|---|
| `base` (`p.a`) | Base Damage | `sum_base_dmg` |
| `base_per_status` (`p.b`) | Base Damage per Status Effect | CO-type per-status bonus in `sum_co` |
| `flat_change_dmg` (`p.z`) | Increase Base Damage by +X | `incarnon_flat` (HP) |
| `abilityCombo` (`p.u`) | Combo Ability Multiplier | T-11 value |
| `crit_chance` (`p.c`) | Critical Chance | `sum_cc_rel` |
| `mult_crit_chance` (`p.d`) | Critical Chance (multiple with others) | factor `(1 + x)` in M_cc of E-5 (inside the CC_min floor, `02` E-2.4); game source of the form: Deadly Maneuvers, "multiplicatively applied after buffs" (`W:Deadly_Maneuvers` L30; ×5 per its rank table, the page example implies ×6, `04` OQ-438) |
| `flat_crit_chance` (`p.e`) | Critical Chance (absolute bonus after mods) | `sum_cc_abs` |
| `crit_chance_weakp` (`p.f`) | Weak Point Increase Critical Chance | `sum_cc_rel` on weak-point hits |
| `crit_mult` (`p.g`) | Critical Multiplier | `sum_cd_rel` |
| `crit_mult_add` (`p.h`) | Critical Multiplier (absolute bonus after mods) | `sum_cd_abs` (before tiers, D-014) |
| `crit_mult_mult` (`p.i`) | Critical Multiplier (multiple after all sources) | `cd_field_mult` factor `(1 + x)` |
| `status_chance` (`p.j`) | Status Chance | `sum_sc_rel` |
| `status_chance_mult` (`p.k`) | Vulnerable to Status Chance | factor `(1 + x)` in `sc_mult` |
| `absolute_status_chance` (`p.l`) | Status Chance (absolute bonus after mods) | `sum_sc_abs` |
| `status_damage` (`p.m`) | Status Damage (as Elementalist) | `sum_status_dmg` |
| `vuln_status_damage` (`p.o`) | Vulnerability Status Damage (as Burning Hate) | entry x in the status-damage vulnerability group V_sd of E-13 (`V_sd = 1 + Σ v_j`, additive with Burning Hate and Pageflight, snapshotted at proc creation; `03` §9.4; the inspiration sums Burning Hate, itself the stat `vuln_status_damage: 1.2`, and the custom stat into one factor `(1 + md)`, `JS L205-209`, `L10467-10469`, `L10498-10502`; no game source for a custom stat, grouping UNVERIFIED, `03` OQ-324) |
| `speed` (`p.p`) | Fire Rate / Attack Speed | fire-rate or attack-speed bucket |
| `multishot` (`p.q`) | Multishot | `sum_ms` |
| `magazineSize` (`p.r`) | Magazine Size Increase | magazine percent bucket |
| `reloadTime` (`p.s`) | Reload Time Increase | reload-speed bucket |
| `mult_for_head` (`p.t`) | Headshot Increase Damage (as Deadhead) | `sum_hs_mult` |
| `headshot_mult` (`p.v`) | Weak Point Increase Damage (as Primary Acuity) | `sum_wp_dmg` (D-012) |
| `initialCombo` (`p.w`) | Initial Combo | initial combo count |
| `dmgVulnerability` (`p.x`) | Damage Vulnerability | multiplicative entry in `vuln` |
| `heatInherit` (`p.y`) | Heat Inherit | Heat DoT elemental bonus of an external first Heat source (D-028) |
| `heatInheritFaction` (ours, companion of `p.y`) | Heat Inherit faction bonus | faction bonus F_inh of the external first Heat source, default 0 (`03` §10.3; "While Faction Damage is inherited, it is only applied once to subsequent procs that inherit it", `W:Damage/Heat_Damage` L84-86) |
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
| `02-crit-status-multishot.md` | E-4 to E-10: crit chance (relative, absolute, floor, Piercing Navigator, combo hooks), crit multiplier (quantization, relative, absolute, field multipliers), tiers and Vigilante, headshots, weak points, head-crit, Sonar, body-part rules for radial and 1×-headshot weapons; status chance (relative, absolute, Tau multiplier, per pellet, Kohm division); proc count and type selection, immunity renormalization, ragdoll Impact; forced procs (weapon traits; stance procs as data from `07`); Hunter Munitions, Internal Bleeding, Hemorrhage, Magnetic Welt; Secondary Encumber, Archon Continuity and Vitality, Primary Debilitate triggers (`02` E-2.17 to E-2.17b); multishot (floor, Acuity lock, beam merge CD-10, speargun throw) | status effects themselves (`03`), body-part data per enemy (`05`) |
| `03-status-effects.md` | every status type: duration, timers, caps (incl. enemy-specific caps CD-07 as rules), replacement; debuff functions (Viral, Magnetic, Corrosive strip, Heat strip ramp, Cold crit damage and Frozen, Puncture crit chance, Tau, Radiation, Impact, Void, Microwave, Knockdown, Lifted); DoT tick formula E-13 (seed, "+1", coefficients, elemental scaling, faction twice, status damage, crit, body part, inheritance list); Heat shared timer and inherit; Blast fuse and detonation; Gas; Electricity; Magnetic break proc; status duration (incl. per-type SD_s) and negative SD; distinct-status counting (what counts, CD-04 timing); ragdoll target state RAG (duration, Impact-discard state; fed by `07`'s crowd-control proc events, used by `02`'s Impact rule) | layer routing of ticks (`05`), expectation of n in EV (`08`) |
| `04-conditional-bonuses-and-buffs.md` | CO family (sources, catalogue per attack with behaviour, ratio and exclusions, `G_F` membership, timing, sequential-pellet counting rule (which earlier instances count, E-4.2), stance ratio formula, CO in DoT seeds); `G` membership and the default rule; condition model (classes of T-02, triggers, stacks, durations, decay); Galvanized and other conditional mods; weapon and warframe arcanes; ability buffs and Helminth values (Strength at the caster); target vulnerabilities and debuff abilities (catalogue; stage math in `05`); auras and Coaction Drift; archon shards; focus; companion and squad buffs; extra hits (E-14) incl. Melee Duplicate; stealth and finishers; riven stat to bucket mapping and the riven value formula; set bonus effects; custom stats (6.3) | legality of builds (`09`), enemy-side formula (`05`), EV expectation of the CO counts (`08`) |
| `05-enemies.md` | enemy data model (incl. status immunities and per-enemy status caps as data); faction damage-type table and overrides; level scaling (NP-2), Steel Path (incl. flat bonuses), Eximus, overguard scaling, party scaling; armor (DR, clamp, floor, E-15 strip combination incl. ability strips, Corrosive Projection, flat base-armor strips, Hydroid); per-type minimum 1; shields (Toxin bypass and exceptions, gate, recharge for TTK); overguard (modifiers, caps, Fortifier); health (Viral stage); vulnerability stage; innate DR; unique DR (attenuation, flat DR, adaptation, Orphix) and T-28; layer order and overflow (CD-08); body-part data and head-crit capability supplied to `02` | status values (`03`), TTK procedure (`08`) |
| `06-ranged-firing-model.md` | the ranged schedule generator used by both evaluators; fire rate (bow ×2, semi-auto cap, Cannonade lock, floor); burst; charge (incl. partial charges); beams (ticks, ammo, ramp); spool (incl. Kohm pellets); shotguns; AoE and falloff (Φ(d), progenitor falloff, Incarnon-flat exception, radial headshot rule); parent and child attacks; magazine, ammo cost, efficiency, infinite ammo; reload (standard, per shell, battery, magazine-1, Vectis, Epitaph); first/last-shot schedule; sniper combo (count, multiplier, decay); kitgun assembly; arch-guns; exalted ranged weapons; companion weapons; Incarnon form (magazine, switch time); duty ρ per class | crit/status math (`02`), CO ratios of child projectiles (`04`) |
| `07-melee.md` | the melee schedule generator; attack speed; stance combos (durations, hit multipliers, physical bonuses, forced procs, CO ratio data) and stanceless defaults; combo counter (gain, tiers, caps, duration, decay, Initial Combo, regeneration); heavy attacks (damage, heavy and combo multipliers, Incarnon-flat exception, crit doubling, HAE, wind-up, Tennokai); slams and heavy slams; slide attacks; glaives; gunblades; whips; zaw assembly; exalted melee and ability combo (T-11); melee-only multipliers in `G` | arcane values (`04`), stealth formula (`04`) |
| `08-evaluation-and-outputs.md` | EV algorithm (section 4.2 in full: stack-count model (E-8.3), independence, window walk, Heat window mean, Little's law); the EV expectation of the sequential-pellet counts n_j and n_COL,j (E-8.10, sole owner; the counting rule is `04` E-4.2); MC simulator (event set and order, RNG choice and keying, two passes); layer views; TTK (layered, horizon, extrapolation, EV approximation); every output of section 4.5 with exact formulas; attack groups and "combine attack effects"; compatibility profile and the oracle procedure (tolerances, oracle-safe scenarios); EV-versus-MC tolerance; optimizer metric definitions (DPS variants, TTK, max hit), not the search | game mechanics (all other documents) |
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
| Ragdoll target state RAG | `03` (§8.10) | `02` (Impact rule), `07` (crowd-control events) |
| Event time grid (CD-14) | `08` (E-8.0a) | `03`, `06`, `07` |
| External statuses (T-06, T-43) | `03` (stack state) | `04` (CO count, n_COL), `08` (`n_ext`) |
| Sequential-pellet CO counts n_j, n_COL,j | rule (which earlier instances of the shot count): `04` (E-4.2; instance order `03` §7.1 P4a, count definition and cap `03` E-3.12); EV expectation: `08` (E-8.10), sole owner (per-instance proc probabilities from `02` E-2.18) | `04` (C, E-4.1), `03` (DoT seeds take the inducing instance's n_j) |
| Elemental DoT membership (`dot_elem`: which elemental sources raise their element's DoT) | rule: `03` (§9.3); per-source `dot_scaling` flags (the rule applied): `04` for the sources it catalogues, `01` for the others (§4.10.5, Table 4.10-B); the sum `elem_dot_sum`: `01` (E-1.15) | `03` (E^dot_s in E-13), E-14 (extra-hit DoT) |

---

## 8. Open questions that block exact accuracy

Each question names the evidence that would settle it. In-game tests use the Simulacrum unless stated; record
results in `research/test-cases/` with the build, enemy, level and observed damage numbers.

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-001 | In Adding-CO attacks, do faction, crit, body part and elemental composition multiply the CO term? | yes (E-11) | Braton Prime with Serration + Galvanized Aptitude (2 stacks) vs a Lancer carrying 1-2 statuses applied by another weapon; compare non-crit and crit numbers with Primed Bane on and off against both hypotheses |
| OQ-002 | Is the CO term inside the quantization scale, and does the Incarnon flat enter the scale? | no: the CO part is multiplied by the quantized composition (sourced: Zhuge Prime catalogue row, "the 'real' in-game value is 48.4375 for both the base damage and aptitude damage", `W:Condition_Overload_(Mechanic)` L371); the Incarnon-flat half is unobservable under D-005's proportional split and reduces to OQ-030 and `01` OQ-102 | see OQ-030 and `01` OQ-102 |
| OQ-003 | Proc count rule for SC > 1: `floor + Bernoulli(frac)`? | yes | single-element weapon at exactly SC 1.5; count stacks added per pellet over 200 shots (expect only 1 or 2) |
| OQ-004 | Are proc weights computed on quantized types? | yes | Nagantaka Prime (Impact share quantizes to 0): 500 hits, any Impact proc disproves |
| OQ-005 | Rounding of fractional status durations (DoT ticks and debuffs) | ticks `floor(6(1 + SD))`; debuffs continuous | Lasting Sting at a rank giving +55 % SD: count Slash ticks (9 expected) and time Corrosive expiry (12.4 s expected) |
| OQ-006 | Same weapon, later Heat procs: faction once (inherit rule) or twice? | twice | Heat-only weapon with Primed Bane vs Grineer: compare the Heat tick before and after the second proc joins |
| OQ-007 | Do Blast-element bonuses scale Blast procs? | no | Blast proc tick with and without a Blast-element source (Valence Formation set to Blast or a riven splice) |
| OQ-008 | Does the 4-stack cap of status-capped enemies limit DoT instances? | yes | Slash procs on an Acolyte or Lich: count simultaneous Bleed numbers above 4 |
| OQ-009 | Overguard level input `L − 1` or `L − BaseLevel`; Steel Path on overguard | `L − 1`; no SP factor | Eximus Heavy Gunner (BaseLevel 8) at level 58: overguard 118,710 vs 105,506; repeat on Steel Path |
| OQ-010 | Eximus base health variable: `L − BaseLevel` or absolute level | `L − BaseLevel` | Eximus Heavy Gunner level 58 health 82,305 (d = 50) vs the absolute-level reading |
| OQ-011 | Overflow at overguard and shield breaks; DoT ticks during the shield gate; DoT on overguard | CD-08 | one-shot tests on a shielded Corpus unit and an Eximus with exact damage numbers |
| OQ-012 | Semi-auto cap value and duplex cadence (Tigris) | semi-auto cap 10 /s; Duplex: two shots (press and release, release gap 0, UNVERIFIED) per 1/FR sequence (`W:Fire_Rate` L8: fire rate caps "how quickly each two-shot sequence can be initiated"; the per-mechanic page beats the DPS-formula assumption of `W:Damage/Calculation` L476 by section 1.4 rule 3; `06` D-608, TV-06-31) | frame-timed fire-rate measurement at high FR with the accessibility auto-fire option; Tigris FR 2, Mag 2: 2 shots per 0.5 s or per 1 s; minimum press-release gap |
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
| OQ-024 | Combination position of the exilus slot, of hypothetical weapon-arcane or Incarnon primary elements and of warframe-arcane elements (Theorem Demulcent); Nourish, Valence Formation, field elements and augments are non-combining since U38 and have no position | exilus after slot 7 (position 8); warframe arcanes at position 10 (`01` OQ-103, OQ-104, OQ-119); the Violet shard is settled (with the innate elements in HCET order, `W:Violet_Archon_Shard` L40-43) | Tenet weapon with Violet shard, innate element and one mod element: observe the combined result; `01` OQ-119 tests for Theorem Demulcent |
| OQ-025 | Electricity and Gas proc headshot behaviour (1× vs Deadhead counted twice) | 1× base, HS bonuses apply once | Electricity proc numbers on headshots with and without Deadhead |
| OQ-026 | Game uses f32 scaling with the documented operation order; armor `floor` | yes | high-level health readout (level 9999) vs the f32 and f64 predictions |
| OQ-027 | Elemental mods scale off the progenitor-inclusive base | yes | Kuva weapon at 60 % valence with one elemental mod: per-type numbers |
| OQ-028 | Do sibling pellets of one shot see each other's Corrosive, Heat, Cold and Puncture procs? | no (Viral analogy) | shotgun first-shot test on a fresh armored target vs single-pellet shots |
| OQ-029 | Complete list of enemies whose shields block Toxin | Treasurer, Hounds | wiki `protoshield` template parameter or in-game checks |
| OQ-030 | How the Incarnon flat splits across damage types | proportional to `Bw_t + V_t` (`01` E-1.3) | Incarnon weapon with mixed IPS: per-type numbers before and after the evolution |

---

## Review notes

Revision after the three-critic review of the whole spec (2026-11-24). Every issue was checked against the cited
sources and the topic documents; the changes below were applied, and the topic documents named in brackets must follow.

- Adopted (EV and crit): crit-only procs carry C1 (§4.2, CD-13); Internal Bleeding, Hemorrhage and Magnetic Welt once
  per instance with π_IB (§2.4, §4.2); live counts = certain forced procs + Poisson with exact per-event presence
  (§4.2; `08` A-3, E-8.2, G-1); steady μ as the arrival mean (§2.4, §4.2; `08` A-10, G-13); `dps_sustained` with
  per-magazine spool-up and the steady ramp mean, sniper combo window mean (§4.1, §4.2; UNVERIFIED metric choice).
- Adopted (formulas): E-4 CDM0c, E-5 M_cc and CC_r, E-6 π_p, E-7 as `02` E-2.10 (headshot bonuses on headshot parts
  only, body and radial branches, part crit factor h), E-10 MS0+ and children; E-11 with ρ_co on multiplying rows, κ,
  B_co^abs and the clamps; E-12 with R_ℓ, R_OG and B_SH; E-13 with c_k and Part_dot inside the seed (`03` E-3.4, E-3.5,
  §15, TV-03-01 and `08` §13 step 5 follow), Sonar inherited (D-095; `03` §9.2, `dot_part`), V_sd and M_tick, SD_s, ε,
  M_OG,p on Slash ticks; E-14 radial and non-standard parents, `elem_dot_sum`, Heat inherit override (`04` §6.2);
  E-15 f_flat and P_pts; S_j and H_j out of G (`04` §2.12 and §3.2, `08` §4.1 S_j field display-only).
- Adopted (rules and register): Piercing Navigator in ΣCC_abs; D-067 ΣCD_abs (`04` §5.12 follows); CD-07 caps per
  `05` §4.4; per-view `og_present` (`03` E-3.2, `05` §18, `08` §4.5 follow); class (u) and the timing-stat rule in
  §6.2; T-22 on for families (CD-12); CD-14 time grid (`03` §7, `06` E-6.11 follow); D-073 three times; D-037 Saryn
  passive on every proc (`04` E-4.14, `08` A-4, E-8.11, G-14 follow); D-005, D-009, D-019, D-021, D-025, D-028, D-034,
  D-052, D-053, D-060, D-064, D-065, D-074, D-082, D-092 restated; OQ-002, OQ-012 (Duplex), OQ-024 restated; T-30 to
  T-43 added, T-03, T-08, T-16, T-18, T-22, T-23, T-25, T-28 reworded; §1.3 scope, §1.5 data gap, §7 owners
  (ragdoll state `03`), NP-2 riven owner `04`.
- Partly rejected: the minor issue on §3.8 step 5 gave the k = 1 Impact value as 21.497877; full precision gives
  21.497876 (4.884687758 × 4.401074743 = 21.4978759), which is printed instead.
- Step 8 of §3.8 now uses the arrival-mean definition of §4.2 (38.84 and 9.24 instead of the time averages 39.04 and
  9.25); states A and B and step 10 are unaffected because state B takes the caps. All §3.8 values were recomputed with
  `python3` (numpy float32 for NP-1 and NP-2).

Follow-up round (2026-11-24), applying the requests other documents made of this one after their revisions:

- `vuln_status_damage` (`03`): §6.3 already put the stat additively in V_sd (round 1); kept, with the inspiration's
  evidence added (`JS L205-209`, `L10467-10469`, `L10498-10502`: Burning Hate is the stat 1.2, summed with the custom
  value into one `(1 + md)`). `03` §9.4 and OQ-324 still describe the old "(1 + x)" row and `04` §10 still has it.
- EV of the sequential-pellet counts (`04`): §3.3, the §4.2 rows Multishot and Distinct statuses, §7.1 (`04`, `08`)
  and a new §7.2 row make `08` E-8.10 the sole owner of E[n_j] and E[n_COL,j]; `04` E-4.2 keeps the counting rule.
- D-067 (`04`): the example was reversed; ΣCD_abs gives CDM 5.401 and ΣCD_rel 6.402 (`02` TV-02-01, rechecked with
  python3: 2.000488519668579 × 2.2 + 1 = 5.401075, × 3.2 = 6.401563).
- Stance formula (`08`): §3.3 now writes the `adding` term as `r_in·κ·B_co,j·C` with `B_co,j = ρ_co·B_ref,j` (fixed
  rows `B_co^abs`), as `04` E-4.6, and gives the heavy base `H_j·(B' − X) + X` of `07` E-7.20b.
- Elemental DoT membership (`01`): new §7.2 row (rule `03` §9.3; flags `04` and `01`; sum `01` E-1.15); the §2.4
  `dot_elem` row now lists `03` §9.3's members instead of "all mods".

Tail round (2026-11-24), settling the requests of `02`, `03`, `04`, `05`, `07`, `08` and `09`; every issue was checked
against the owner documents and the cached wiki pages and all eight were applied:

- E-7, §2.4 `headcrit`, T-03 (`02` §8.1 proposals): the `non_body` branch Part_B = 1 (`W:Enemy_Body_Parts` L37), the
  `radial_parent_part` radial part (Zymos, `W:Zymos` L34) and `torso_only` (Peacemaker, `W:Peacemaker` L84) now match
  `02` E-2.10 (TV-02-05: Symbilyst shield with Meticulous Aim 1, not 0.55; Zymos head explosion Part 3, not 1).
- E-14: the extra-hit DoT now restates `03` §9.8 in full (V_sd,p, M_tick(τ), M_OG,p), so its tick equals `04` E-4.14
  (TV-04-11: 54.15, × 2.2 = 119.13 under Burning Hate) and `08` E-8.11's "E-14 tick" needs no change.
- D-064 → `07` E-7.39 and D-065 → `07` E-7.38 (`07` OQ-739); §4.1 layer views add the overguard ragdoll immunity
  (`03` E-3.13, `08` E-8.27); CD-07 and D-053 give Kuva Liches Impact 5 (`03` §6.1, `05` §4.4; `W:Bosses` L925,
  `W:Damage/Impact_Damage` L22, L325-330); T-23 refs complete.
- T-37 covers T-13 Roar only; T-14 … T-16 are always self-cast (adopted from `09` OQ-926; `W:Roar` L14 is the only
  page that reaches allies). `04` §4.4 (E-4.10 and the T-37 paragraph) and `09` §10 (and its symbol and D-910 rows)
  must follow.
- §3.8 step 8 and §4.2 "Live stacks" use `08` E-8.3's random part (exact P(N = 0), mean μ, zero-truncated shape);
  recomputed with python3: step 8's rounded values are unchanged (PUN P(n = 5) 0.952557, E[min(n, 5)] 4.928436).
- §4.2 Steady: the time-average overstatement is now quoted against `08` E-8.3's count at the arrival (+4.72 %,
  `08` TV-08-17; recomputed with python3: 246.2563675 / 235.1551593); the former +6.45 % compared with a plain Poisson
  count at the arrival mean (231.3416973), which E-8.3 superseded; G-13 is retired.
