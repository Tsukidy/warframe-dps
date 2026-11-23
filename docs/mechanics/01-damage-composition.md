Status: draft (spec)

# Damage-calculation spec, part 01: damage composition

This document turns an attack's listed base damage into the quantized per-type composition `ratio_q[t]` and the
two scalar buckets that sit next to it: the base-damage bucket `sum_base_dmg` (A) and the faction bucket
`sum_faction` (F). It owns stages ST-1 to ST-3, the base-damage part of ST-4 and the faction part of ST-5. It follows
[00-overview.md](00-overview.md) (notation, symbols, IDs, CD-02 and E-1 to E-3); where this document adds detail to
an overview rule it says so, and where it would change an overview decision it proposes the change in
[section 9](#9-open-questions).

Related documents: [00-overview.md](00-overview.md) (master), `02-crit-status-multishot.md` (proc weights read
`ratio_q`), `03-status-effects.md` (DoT seeds, `dot_elem`, faction twice on DoT), `04-conditional-bonuses-and-buffs.md`
(CO term, `G_F`, `G`, effect values and conditions), `05-enemies.md` (faction data, type modifiers),
`06-ranged-firing-model.md` (falloff Φ(d), kitgun assembly), `07-melee.md` (stance data, zaw assembly, heavy
attacks), `08-evaluation-and-outputs.md`, `09-build-rules.md` (slots, riven legality, Incarnon tiers, valence
fusion).

Contents:

- [1. Scope and stages](#1-scope-and-stages)
- [2. Inputs](#2-inputs)
- [3. Outputs and symbols added](#3-outputs-and-symbols-added)
- [4. Rules and formulas](#4-rules-and-formulas)
- [5. Worked example](#5-worked-example)
- [6. Edge cases](#6-edge-cases)
- [7. Inspiration differences](#7-inspiration-differences)
- [8. Test vectors](#8-test-vectors)
- [9. Open questions](#9-open-questions)

Citation keys are those of the overview §0.2. `W:<Page>` line numbers refer to the raw copy in
`research/external/cache/wiki/` (`/` → `__`). Pages added to that cache for this document: `Chromatic_Blade`,
`Reactive_Storm`, `Comet_Rounds`, `Smite_Infusion`, `Fireball_Frenzy`, `Shock_Trooper`, `Freeze_Force`,
`Venom_Dose`, `Thermal_Transfer`, `Sentient_Incision`, `Kuva_Nukor`, `Tenet_Glaxion`, `Alchemy`,
`The_Perita_Rebellion`, `Decree`, `Summoner's_Wrath`, `Paracesis`, `Sacrificial_Pressure`, `Gale_Kick`.
Every number in this document was computed with `python3` (reference implementation of sections 4.2-4.13, kept
outside the repository; the 48-cell HCET table check of TV-01-23 was run against the rendered wiki table).

---

## 1. Scope and stages

Scope (overview §7.1, row `01-damage-composition.md`): ST-1 to ST-4 except the CO term: per-attack base damage per
pellet; exalted base scaling by Strength; kitgun and zaw base damage as delivered by `06`/`07`; progenitor bonus
(value, fusion range only as input); Incarnon flat damage (split, quantization scale, falloff exception data);
damage-type conversion mods; physical mods; elemental mods; element combination order (slots, exilus position,
riven bottom-up order, innate last, HCET, non-mod and non-combining sources: Nourish, field elements, Valence
Formation, Unairu Impact, Phoenix Talons arithmetic); quantization (E-3, CD-02); base-damage bucket membership;
faction bucket (members, which bane applies to which enemy faction and sub-faction).

Not in scope: the CO term, `G_F` and `G` (`04`), proc weights (`02`), the falloff curve Φ(d) (`06`), stance data
(`07`), kitgun and zaw assembly (`06`, `07`), effect values and condition states of individual mods, arcanes and
buffs (`04`), build legality (`09`).

| Stage | Owned here | Shared with |
|---|---|---|
| ST-1 Base vector | all (E-1.1 to E-1.4) | inputs from `06` (kitgun), `07` (zaw) |
| ST-2 Composition | all (E-1.6 to E-1.9) | stance physical `S^phys_t` from `07`; Phoenix Talons value and every effect value from `04` |
| ST-3 Quantization | all (E-1.10) | — |
| ST-4 Base scalar | the factor `B'·(1 + A)` (E-1.11) and the membership of A (E-1.5) | CO term, behaviour and `G_F` (`04`) |
| ST-5 Weapon-side multipliers | the factor `(1 + F)` and its membership (E-1.12) | `G` (`04`, `06`, `07`) |

---

## 2. Inputs

| Input | Symbol | Unit | Owner | Notes |
|---|---|---|---|---|
| Attack list of the weapon; per attack the listed base damage per pellet, by type | `base_listed[t]`, B0_t | HP | data pipeline (`docs/data/`); kitgun base `06`, zaw base `07` | per pellet, per beam tick, per melee hit before the stance multiplier (`DS:wiki` §3 "`Damage` is a per-pellet map", Hek 75 × 7; `DS:wfcd` "the attack `damage` is per pellet") |
| Exalted Strength model of the weapon | — | enum | data (`06`, `07`) | `base_x_str` (model 1), `mult_in_A` (model 2, with per-rank m), `peacemaker` (model 3), `none` (section 4.3) |
| Caster Ability Strength | `strength`, Str | factor | T-20; caster value per `04` | |
| Progenitor element and fraction | `progenitor_type` e_v, `progenitor_frac` v | DamageType, fraction | T-24 (legality and fusion range `09`) | v ∈ [0.25, 0.60] |
| Incarnon flat damage per attack | `incarnon_flat` X | HP per pellet | value and attack filter `04`; tier choice `09` | 0 when no such perk is selected or its condition is inactive |
| Build layout | — | — | `09` | slot index 0-7 for the 8 weapon mod slots in reading order, 8 = exilus; riven stat rows in display order |
| Active contributions (each effect's bucket here, value and condition state from `04`, T-02) | — | — | `04` | one record per effect: bucket, element, value, condition active or not |
| Stance physical bonus of the current hit | `stance_phys[t]`, S^phys_t | bonus | `07` | 0 outside stance hits |
| Phoenix Talons value | pt (`phys_talons_frac`) | fraction | `04` (Focus catalogue), T-19 | 0.30 at max rank |
| Chromatic Blade element; Reactive Storm equipped; Crucible Overflow element; Valence Formation element | — | DamageType | T-25 and `04` | |
| Target faction f (affiliation) and the layer being hit | — | enum | `05` | f for F (E-1.12) and for weakness-element effects (section 4.10.4) |

---

## 3. Outputs and symbols added

Outputs that use overview symbols (overview §2.4): `base[t]` (B'_t), `base_total` (B'), `base_ref` (B_ref),
`progenitor[t]` (V_t), `incarnon_flat[t]` (X_t), `str_base_mult` (m_str), `sum_base_dmg` (A), `sum_phys[t]` (P_t),
`sum_elem[t]` (E_t), `ratio[t]` (r_t), `ratio_q[t]` (q_t), `sum_faction` (F).

**Symbols added** (overview §0.3: domain-word prefixes `base_`, `elem_`, `phys_`, `comp_`, `faction_`):

| ID | Symbol | Unit | Meaning | Consumers |
|---|---|---|---|---|
| `base_weapon[t]`, `base_weapon_total` | Bw_t, Bw | HP | `B0_t·m_str` after base conversions (Chromatic Blade, E-1.1b); the part of B' that follows the weapon's own falloff | `06` (falloff), `07` (heavy) |
| `progenitor_total` | V | HP | `Σ_t V_t = v·Σ_t B0_t` | `06` (progenitor falloff) |
| `incarnon_flat_total` | X | HP | the Incarnon flat of this attack (equals the overview's scalar `incarnon_flat`) | `04` (CO), `06` (falloff exception), `07` (heavy exception) |
| `elem_contrib` | — | list | every active element or Impact-like contribution: {source, position key, type, value, class ∈ {combining, innate_group, noncombining}, dot_flag} (section 4.10) | `03` (`dot_elem`), UI |
| `elem_seq` | seq | list of ≤ 4 PE | de-duplicated combine sequence of primary elements (E-1.8) | UI, tests |
| `elem_group[f]` | G_f | set of PE | primaries that form final type f (a pair, a single leftover, or empty) | `03`, UI |
| `elem_mod_sum[p]` | ê_p | bonus | Σ of every combining and innate-group contribution of primary p, combined or not | `03` (`dot_elem` for PE, forced procs of extra hits) |
| `elem_noncomb[t]` | n_t | bonus | Σ of non-combining contributions of type t (section 4.10.3) | `03`, UI |
| `phys_talons[t]` | PT_t | dimensionless | Phoenix Talons share added to physical type t (E-1.7) | UI |
| `comp_sum_q` | Σq | dimensionless | `Σ_t q_t` | `02`, `08`, tests |
| `base_y0` | Y0 | HP | `B'·max(0, 1 + A)` (E-1.11): the base scalar before `04` adds CO and `G_F` | `04` (E-11) |
| `faction_group(f)` | — | enum | bane group of target faction f (Table 4.15-B) | UI |

`sum_elem[t]` (E_t) is defined here as the bonus part of r_t for elemental t: `E_t = Σ_{p ∈ G_t} ê_p + n_t`, so that
the overview's `r_t = B'_t/B' + E_t` holds with `B'_t` read as the summed innate base of the group (E-1.9).

---

## 4. Rules and formulas

### 4.1 Order of operations

For one attack a (and, where noted, one stance hit or one target layer), the engine computes, in this order:

1. Read B0_t (E-1.0). Kitgun and zaw parts are already assembled by `06`/`07`.
2. Exalted Strength scaling: `Bw_t = B0_t·m_str` (E-1.1).
3. Base conversions that act on base damage (Chromatic Blade; generic `convert_base` hooks) (E-1.1b).
4. Progenitor bonus V_t from the listed total (E-1.2).
5. Incarnon flat split X_t (E-1.3).
6. Base vector B'_t, B', B_ref (E-1.4).
7. Collect the active contributions of the build (T-02 state from `04`): A (E-1.5), P_t (E-1.6), pt (E-1.7),
   element contributions (section 4.10), F (E-1.12).
8. Element combination: sequence, pairs, groups (E-1.8).
9. Composition r_t (E-1.9): physical step (P_t, S^phys_t, Phoenix Talons), conversions that act after physical mods
   (Reactive Storm), element groups, innate secondaries and other types, non-combining additions.
10. Quantization q_t (E-1.10), once per final type.
11. Base scalar part Y0 = B'·max(0, 1 + A) (E-1.11), handed to `04` for E-11.
12. Faction bucket F for the target's faction (E-1.12), used at ST-5 (and again by `03`, `04`).

Steps 9-10 are repeated per stance hit when `S^phys_t` differs between hits (EC-01-34), and per target layer when a
layer-dependent effect is active (Reactive Storm, Sentient Incision: EC-01-38, EC-01-39). Steps 7-10 are repeated
whenever a conditional contribution changes state in the `simulated` mode of T-02 (EC-01-53). All arithmetic is f64
(overview U-4); sums over types are taken in `DamageType` enumeration order.

### 4.2 Per-attack base damage per pellet

```text
E-1.0  B0_t = base_listed[t] of attack a, per pellet (per beam tick for continuous attacks, per hit for melee)
```

- Every attack of a weapon (normal, charged, alt-fire, Incarnon Form, AoE child, slam) has its own B0 and is composed
  on its own; mods apply to each attack on its own base (`F:weapon` §10.1; overview CD-12).
- Kitgun B0 is the chamber's per-grip damage vector (`Module:Modular/data` `Damage[grip]`, `F:weapon` §11); zaw B0
  is the assembled strike (+ grip, + link) vector from `07`. This document consumes them unchanged.
- An attack whose damage map is empty or sums to 0 (WFCD: Haalvu, Simulor and Synoid Simulor "Orb Launch", Shadow
  Clones, `DS:wfcd` §2) has no composition: the engine reports a data error for that attack and excludes it
  (EC-01-13).

### 4.3 Exalted base scaling by Strength

Three models exist (`F:weapon` §14, `F:melee` §13):

```text
E-1.1  model 1 (base_x_str):  m_str = Str                  Exalted Blade, Valkyr Talons, Iron Staff, Desert Wind,
                                                            Shadow Claws, Diwata, Artemis Bow, Balefire Charger,
                                                            Dex Pixia, Noctua, Glory (data flag decides)
       model 2 (mult_in_A):   m_str = 1;  A += m(rank)·Str − 1          Neutralizer (Cyte-09), Lizzie (Temple)
       model 3 (peacemaker):  m_str = 1;  A += 1.5·Str (rank 3 value)   Peacemaker (Mesa)
       none:                  m_str = 1                    every other weapon, including Garuda Talons
       Bw_t = B0_t · m_str
```

Sources: model 1 `W:Razorwing` ("Ability Strength modifies the base damage properties of the weapon, damage
bonuses like Hornet Strike … are fully multiplicative with … Ability Strength"), `F:melee` §13 ("Base damage scales
with Ability Strength"; Garuda Talons "do not scale with Ability Strength"); model 2 `W:Neutralize` line 19 and
`W:Exalted_Solo` line 17 ("Damage multiplier is additive to base damage mods such as Serration and Galvanized
Aptitude"; m = 1, 1.08, 1.16 (1.15 for Lizzie), 1.25 by ability rank); model 3 `W:Peacemaker` via `F:weapon` §14
(`50 × ramp × (1 + 1.5 × Strength + DamageMods) × (1 + ElementalMods)`). The form `m·Str − 1` of model 2 is a
Decision (a 1× multiplier at 100 % Strength must not add damage), UNVERIFIED (OQ-109; inspiration difference D-104).
Str is the caster's Ability Strength (T-20).

### 4.4 Base conversions

```text
E-1.1b Chromatic Blade (Exalted Blade, every attack except Slam and Heavy Slam):
         Bw_c += Σ_{t∈IPS} Bw_t ;  Bw_t = 0 for t ∈ IPS          c = element chosen by energy colour (T-25)
       generic convert_base(a → b, fraction k) hook:  Bw_b += k·Bw_a ;  Bw_a −= k·Bw_a      (adds, never overwrites)
```

- Chromatic Blade: "elemental damage replaces the innate physical elements and will combine with any elemental
  damage modded on Exalted Blade"; "Physical damage mods will no longer affect Exalted Blade's damage"; "does not
  affect Exalted Blade's slam attacks" (`W:Chromatic_Blade` lines 38-44). Because it replaces base damage, c is an
  innate element (innate group of E-1.8).
- The PvE game has no mod that converts one physical type into another: the "X % of Damage converted into
  Impact/Puncture/Slash" mods (Comet Rounds, Ripper Rounds, Serrated Edges, Crash Shot …) are Conclave-exclusive
  (`W:Comet_Rounds` line 3; `Module:Mods/data` `Conclave = true`, internal path `/PvPMods/`), and Conclave is out of
  scope (overview §1.3). The generic hook exists for data-driven effects; its rule (the converted amount adds into
  the target type) is the decision of D-009 (UNVERIFIED, no game source).
- Reactive Storm converts after physical mods and is therefore specified in E-1.9, not here.

### 4.5 Progenitor (valence) bonus

```text
E-1.2  V_t = v · Σ_u B0_u   if t = e_v,  else 0          (per attack, per pellet)
       V   = Σ_t V_t                                      e_v ∈ {IMP, HEA, COL, ELE, TOX, MAG, RAD}, v ∈ [0.25, 0.60]
```

- Value: "25 %-60 % of the weapon's base damage", "applies as weapon base damage, meaning elemental mods and status
  that scale from base / modified base damage will be affected" (`W:Kuva_Weapons`, `W:Tenet_Weapons`,
  `W:Coda_Weapons` § Elemental Bonus via `F:core` §3.2). Numeric check of the multiplier on the listed total: Kuva
  Drakgoon charged 460 → 736, Tenet Detron 260 → 416, Coda Hema 52 → 83.2 at 60 % (`F:core` §3.2).
- V is computed per attack from that attack's listed total (inspiration does the same for every attack,
  `addPElement`, JS L12885-12901). It applies to AoE child attacks and slams too: Decision, UNVERIFIED for slams and
  AoE children (OQ-111).
- V is part of B' (quantization scale, E-1.10), of the base that elemental mods multiply (overview OQ-027), of B_ref
  (CO base, `04`; "included … since U31.1.7-U31.5", `F:addmul` §1.7) and of the base that Phoenix Talons uses.
- Combine position: a primary e_v is an innate element (innate group, HCET, E-1.8); MAG and RAD are innate
  secondaries; IMP is a physical type (physical mods and Phoenix Talons weights apply). If e_v equals a type the
  attack already has, the two simply add (same type).
- Falloff: the progenitor part always falls off linearly 100 % → 0 % over the weapon's falloff range ("regardless of
  what the weapon's falloff damage scaling is", rendered `Tenet_Detron.parsed.html`). `06` owns Φ and uses
  `progenitor_total` from here.
- v itself (25-60 %, fusion rounding, 58 % → 60 % snap) is an input (`09`, `F:mods` §10).

### 4.6 Incarnon flat damage

```text
E-1.3  B_ref = Σ_t (Bw_t + V_t)
       X_t   = X · (Bw_t + V_t) / B_ref                    (Σ_t X_t = X exactly; X_t = 0 for every t when B_ref = 0)
```

- X is the perk's "Increase Base Damage by +X" (`F:core` §3.1), **per pellet**: Bronco Prime Incarnon Form (34 per
  pellet × 7 pellets = 238) reads 448 with the +30 perk (= 7 × 64), Vasto Prime Incarnon (70 × 6 = 420) reads 564
  (= 6 × 94), Lato Vandal Incarnon (76 × 2 = 152) reads 174 (`W:Condition_Overload_(Mechanic)` catalogue lines 411,
  501, 463; per-pellet bases from `Module:Weapons/data`).
- Which attacks receive X, under which condition, and its per-variant value (Furis 28, MK1-Furis 34; "+30 with
  Overshields"; Lex "+80 for 8 s on Shield Break") are perk data owned by `04`/`09` (`F:mods` §9, `DS:wiki` §3).
  The catalogue row "Furis 100 / 128" shows the Furis flat also applies to its Incarnon Form attack.
- Split: proportional to `Bw_t + V_t` is the overview decision of D-005 (UNVERIFIED, OQ-030). Consequence used in
  E-1.10: `B'_t / B' = (Bw_t + V_t) / B_ref`, so the composition ratios do not depend on X.
- Exported for other documents: X (`incarnon_flat_total`) and B_ref. `04` excludes X from the CO base on Adding
  attacks (`F:addmul` §1.7). `06` needs the parts Bw, V, X because the flat ignores falloff (bug, `W:Damage_Falloff`
  § Bugs via `F:weapon` §10.2). `07` needs X because the flat is not multiplied by the heavy-attack type multiplier
  but is multiplied by the combo multiplier (`F:melee` §4).

### 4.7 Base vector

```text
E-1.4  B'_t = Bw_t + V_t + X_t                 B' = Σ_t B'_t = B_ref + X
```

This is overview E-1 with `B0_t·m_str` replaced by Bw_t (identical unless a base conversion of E-1.1b is active).

### 4.8 Base-damage bucket A

```text
E-1.5  A = Σ (values of every active member below, for this attack)          additive; negative entries allowed
```

General rule: every **weapon mod or riven stat** whose effect line is "+X % Damage" or "+X % Melee Damage" (negative
values included) is a member (`F:core` §2: `Calculating Bonuses` § Additive Stacking, internal type
`STACKING_MULTIPLY`; § Opportunity Cost item 5 for negative entries). A non-mod source is a member only if it is
listed in Table 4.8-A; any other non-mod "+X % damage" source takes `04`'s default bucket (overview §3.4: an unlisted
multiplier defaults to G).

Table 4.8-A. Members of A.

| Member | Applies to | Source | Status |
|---|---|---|---|
| Serration, Amalgam Serration, Heavy Caliber, Hornet Strike, Augur Pact, Point Blank, Primed Point Blank, Pressure Point, Primed Pressure Point, Sacrificial Pressure (damage part), Spoiled Strike, Rubedo-Lined Barrel, Cannonade mods, Spectral Serration (conditional), negative entries (Hollow Point, Shrapnel Rounds, Frail Momentum, Vile Acceleration, Anemic Agility) | per mod class | `F:core` §2; `W:Cannonade_Mods` ("large increase to base damage") | sourced |
| Riven Damage / Melee Damage (positive or negative) | the riven's weapon | `F:addmul` §11; `F:mods` §8.4 note | sourced |
| Damage arcanes: Arcane Fury, Arcane Rage, Arcane Precision, Arcane Awakening, Arcane Primary Charger, Arcane Blade Charger, Primary/Secondary Merciless, Primary/Secondary Deadhead (damage stacks), Primary/Secondary Dexterity; Arcane Arachne on guns only | per arcane | `F:addmul` §5; `F:buffs` §7; `F:core` §2 | sourced; Arcane Rise UNVERIFIED (page empty, analogy with Awakening) |
| Damage auras: Steel Charge, Rifle Amp, Pistol Amp, Shotgun Amp, Dead Eye, Holster Amp | per aura class | `F:buffs` §4 | sourced |
| Vigorous Swap; Provoked | weapons | `F:buffs` §8; `W:Universal_Weapon_Bonuses` § Damage | Vigorous Swap sourced; Provoked UNVERIFIED (listed only) |
| Ability buffs: Vex Armor, Elemental Ward (Toxin), Octavia Amp, Metronome Forte (melee), Deathbringer, Shooting Gallery, Metamorphosis (Day, ranged only), Rift Torrent, Kinetic Plating with Redline (melee +100 %) | per buff | `F:buffs` §1.6-§1.18; `F:addmul` §4 | sourced; Absorb UNVERIFIED (listed only, `F:buffs` §1.18) |
| Passives: Valkyr Rage (melee), Excalibur Swordsmanship (+10 % swords) | per passive | `F:buffs` §1.20 | sourced |
| Focus: Void Strike (not archguns), Lethal Levitation | Warframe weapons | `F:buffs` §6; `W:Focus/Madurai` line 49 | sourced |
| Heavy-only melee damage: Killing Blow, Melee Assimilation, riven "Melee Damage on Heavy Attack" | heavy attacks and heavy slams only | `F:melee` §4 | sourced |
| Primary Compression | the hitscan-AoE weapons of its "Adds" table (Ambassador, Mausolon, Braton and Burston Incarnon, Ferrox, Tenet Ferrox, Opticor, Opticor Vandal, Trumna, Trumna Prime); on projectile AoE it is a `G_F` factor (`04`) | `F:addmul` §5 (`W:Primary_Compression`) | sourced |
| Primary Plated Round (`15·sqrt(5·Mag)` % scaled by the fraction reloaded) | primaries | `F:weapon` §9.5 ("additive to Serration") | `F:addmul` §5 calls the bucket UNVERIFIED; Decision: A |
| Exalted model 2 and 3 terms (E-1.1) | those weapons | section 4.3 | model 2 form UNVERIFIED |
| Custom stat `base` (T-26, `p.a`) | the build | overview §6.3 | engine accepts negative values (D-106) |
| Decree "+X % damage per status-affected enemy" (mode-specific) | weapons | `W:Decree` lines 237-239 ("additive to … Serration and Hornet Strike") | sourced |

Not members (common confusions): the CO family (`04`); Eclipse, Garuda passive, Parasitic Link, Overdriver,
Duality, Furious Javelin, Hall of Mirrors, Longbow Sharpshot, projectile Primary Compression, Dragon Key (`G_F`,
`04`); Primed and Charged Chamber, Synth Charge, Vauban passive, Navigator, Mecha Empowered, Slam Damage mods, heavy
type and combo multipliers, Braton Incarnon "+Y damage on multishot pellets", Torid Plentiful Mayhem (`G`, `04`,
`07`); faction sources (F); Phoenix Talons (E-1.7); elemental and physical bonuses (E-1.6 to E-1.9).

Melee projectiles (glaive throws and explosions, Exodia Contagion) have a second "weapon damage" bucket (Arcane
Arachne, Vigorous Swap, Holster Amp, Toxin Ward, Power Throw; Vex Armor, Deathbringer, Octavia Amp, Rift Torrent,
Lethal Levitation count in both). That bucket is `04`'s (overview §3.1a); every member above carries a
`universal_weapon_damage` flag so `04` can move it (`F:addmul` §12).

### 4.9 Physical bonuses, stance physical, Phoenix Talons

```text
E-1.6  P_t = Σ (active physical bonuses of type t)        t ∈ IPS; physical mods, riven IPS, Carnis/Jugulus/Saxum
                                                          mods, any catalogue entry of bucket "phys"
       phys_t = (B'_t / B') · (1 + P_t) · (1 + S^phys_t)   t ∈ IPS
```

- Physical mods scale only their own type and are relative to the base share; a type the attack lacks stays 0
  (`F:core` §1, §4: "Physical damage cannot be added to a weapon that lacks that type"; Amprex + Fanged Fusillade).
- Stance physical bonuses multiply with physical mods: `Base Physical × (1 + Stance Physical Bonus) × (1 + Modded
  Physical Bonus)` (`F:melee` §2.1, `W:Stance` § Notes).

```text
E-1.7  Phoenix Talons (pt from 04; 0.30 at max rank):
       w_t   = max(0, B'_t · (1 + P_t))                    t ∈ IPS
       PT_t  = pt · w_t / Σ_{u∈IPS} w_u    if Σ w > 0, else PT_t = 0
       phys_t ← phys_t + PT_t
```

Source: "Physical Damage bonus is calculated by multiplying all of the weapon's base damage (including physical and
elemental damage types) by 0.3, and subsequently multiplying the result by the respective base physical damage type
weighting amongst all base physical damage types present … then added as a flat bonus after physical mods";
"Other physical damage increases (e.g., Jagged Edge) or decreases (e.g., Riven Mods) affect the weighting"; works on
Exalted Weapons and Necramechs, not on abilities (`W:Focus/Madurai` lines 26-38; `F:buffs` §6). In ratio form "all
of the weapon's base damage" is `B'` (Decision: includes V and X), i.e. pt. Decisions (UNVERIFIED, OQ-108): the
weights use `(1 + P_t)` but not the stance bonus; PT_t is not multiplied by `(1 + S^phys_t)`; Unairu Impact
(E-1.9) does not count as base physical; PT is scaled by the base-damage bucket because it becomes part of r_t
(overview D-007, OQ-020). An attack with no physical type gains nothing (Void gets nothing, `F:buffs` §6).

### 4.10 Element contributions

Every active effect that adds damage of an elemental type (or Impact "like an element") is one record of
`elem_contrib` with a class:

#### 4.10.1 Combining contributions (take part in the pairing)

Primary elements (HEA, COL, ELE, TOX) from: weapon mods in the 8 slots and the exilus slot (dual-stat elemental mods
included), riven elemental stats, and any primary-element effect in a weapon arcane or Incarnon perk slot (none is
known in current data; position Decision below). Value: the effect's bonus fraction (`04`). Elemental bonuses are a
fraction of the **total** base B', "always applied at 100 % value, even if the element is completely new" (`F:core`
§5.1, `W:Calculating_Bonuses`, `W:Damage` § Physical Damage).

Position key (combine order), ascending:

| Position | Content | Source |
|---|---|---|
| 0 … 7 | weapon mod slots in reading order: top row left → right (0-3), bottom row left → right (4-7) | `W:Damage` § Modding lines 190-197 ("closest to top left (first) … to the bottom right (last)"); `W:Calculating_Bonuses` line 230 ("the slots are ordered from left to right, top row then bottom row, with any inherent elemental damage … added last") |
| 8 | exilus slot | Decision (overview OQ-024: "exilus after slot 8"), UNVERIFIED; no current exilus weapon mod adds an element (scan of `Module:Mods/data`: only Truth's Flame matches, and only in its curse text), so the choice has no effect on today's data |
| 9 | weapon arcane slot, then Incarnon evolution perks in tier order | Decision, UNVERIFIED (no game source; same relative order as the inspiration's slot indices 10, 18-22, JS L5449-5452, L8606-8629) |
| 10 | innate group (section 4.10.2) | `W:Damage` § Modding line 191 ("Innate weapon elemental damages are considered the very last") |

Inside one position, a riven's elemental stats are taken **bottom-up**: the riven's primary-element rows in reverse
display order (`W:Riven_Mods` § Notes lines 1527-1530: "the elements listed are applied in bottom-up order. An
elemental damage mod installed before the Riven mod will interact with the last element listed … installed after …
will only interact with the Riven if it has 1 or 3 elements, and will combine with the first one listed. This
applies to the Valence bonus … as well"; `W:Damage` § Modding line 203 "priority will be given to the last elemental
stat listed").

A contribution occupies its position only while its value is > 0 (an inactive conditional element does not hold a
place): Decision (EC-01-53, D-105).

#### 4.10.2 Innate group

The innate group holds the primaries that are part of the attack's base or behave as such:

- every primary p with `B'_p > 0` (the weapon's innate element, a primary progenitor e_v, a Chromatic Blade element,
  the Incarnon flat's share of an innate element);
- ELE when Violet Archon Shard Electricity is active, with bonus value
  `Σ_{violet shards with this bonus} (base_s + step_s · N)`, N = number of Crimson + Azure + Violet shards (shard
  itself included), base/step 0.30/0.10 (Tauforged 0.45/0.15) (`F:buffs` §5). "The order of element combining works
  the same as if the electricity damage was innate to the weapon … the weapon's innate element, valence element (for
  adversary weapons), and violet shard's electricity element are all added/combined in HCET order"
  (`W:Violet_Archon_Shard` lines 40-43; `W:Archon_Shard` line 390 "Electricity will combine with Mods").

Members are appended in **HCET** order (Heat > Cold > Electricity > Toxin): "whichever of the two primary elements
comes first in this element order … will be placed second to last in the element combine order, while the other …
last" (`W:Damage` § Modding line 191); "Mod slot 1 → … → Mod slot 8 → Innate Heat → Innate Cold → Innate Electricity
→ Innate Toxin" (rendered `Tenet_Detron.parsed.html`, transcluded from `Adversary System/Progenitor`). Innate
secondaries (BLA, RAD, GAS, MAG, VIR, COR in B') never enter the sequence: they "will always have that damage type",
primaries "cannot combine with a weapon's already combined innate damage type" (`W:Damage` § Modding line 205).

#### 4.10.3 Non-combining contributions (added to their type after combination)

| Source | Type and value (value owner `04`) | Source of the non-combining rule |
|---|---|---|
| Nourish | VIR | "Think of it like adding a +75 % Viral mod"; already combined (`W:Nourish` line 20; `F:buffs` §1.5) |
| Valence Formation | chosen element (any, incl. primaries) | "added parallel to the weapon's Elemental Damage, meaning it will NOT combine" (`W:Valence_Formation` line 29) |
| Fireball Frenzy, Shock Trooper, Freeze Force (HEA, ELE, COL); Thermal Transfer (HEA field, COL field; both → BLA = sum) | primary or BLA | U38: "Elemental Damage from the Thermal Transfer and Valence Formation Augments will now be added parallel … This also applies to Mods that apply Elemental damage to Ally weapons, like Fireball Frenzy … changed for relevant Decrees and Saya's Visions Blessings with 37.0.7" (`W:Fireball_Frenzy` lines 44-48); `W:Thermal_Transfer` lines 27-28 |
| Venom Dose (COR), Smite Infusion (RAD) | secondary | "Does not merge with Toxin or Electricity" / "Heat or Electricity" (`W:Venom_Dose` line 34, `W:Smite_Infusion` line 33) |
| Electric Shield (ELE +0.5 per shield passed, not Strength-scaled), Energy Shell (HEA), Conductive Sphere (ELE), weapon-made fields | primary | "does not combine with other elemental types" (`W:Electric_Shield` lines 16-18; `F:buffs` §1.12) |
| Plunder (COR), Melee Exposure (COR), Crucible Overflow (the crucible's secondary, +1.0) | secondary | secondaries never pair (`F:buffs` §1.22, §8; `W:Alchemy` line 32) |
| Riven spliced elements (BLA, COR, GAS, MAG, RAD, VIR) and secondary-element mods (Magnetic Might, Accelerated Isotope, Atomic Fallout, Radiated Reload, Magnetic Capacity, Magnetic Strafe, Focus Radon, Leaded Gas, Biotic Rounds, Damzav-Vati, …) | secondary | secondaries never pair (`F:mods` §8.4; `Module:Mods/data`) |
| Decree element additions (e.g. "Melee attacks are imbued with fire") | primary | "Heat damage does not combine with modded elements" (`W:Decree` line 159) |
| Custom stat `heatAdd` (T-26) | HEA | overview §6.3 |
| Sentient Incision (weakness element, section 4.10.4) | primary or secondary | Decision for primaries (UNVERIFIED, OQ-110): same as Reactive Storm |
| Reactive Storm (converted Impact, E-1.9) | primary or secondary | "If damage becomes a primary element, it will not combine with other modded primary elements" (`W:Reactive_Storm` line 30) |
| Unairu Initiative | IMP, +1.0 per completed order | "calculated like an Elemental mod bonus … scales with the weapon's total base damage … allows it to affect weapons with no innate Impact" (`W:The_Perita_Rebellion` lines 128-130; `W:Universal_Weapon_Bonuses` lines 87-90) |

`dot_flag` (data for `03`, which owns `dot_elem`): Electric Shield ELE and Sentient Incision do **not** raise their
element's status damage (`W:Electric_Shield` line 18, `W:Sentient_Incision` line 46); augment elements do (U38
"Fixed Elemental Augment Mods not boosting Status Effect damage", `W:Fireball_Frenzy`; `W:Thermal_Transfer` line
36: Heat yes, Blast no); Valence Formation adds to its element's DoT except Blast (`W:Valence_Formation` lines
31-33).

#### 4.10.4 Weakness-element map (Reactive Storm, Sentient Incision)

Table 4.10-A, from `W:Reactive_Storm` lines 34-45 and `W:Sentient_Incision` lines 28-38 (identical except the
overguard row):

| Target | Element |
|---|---|
| Infested | HEA |
| Anarchs | ELE |
| Narmer | TOX |
| Grineer, Kuva Grineer, Zariman, Scaldra | COR |
| Corpus, Corpus Amalgam, Techrot | MAG |
| Orokin | VIR |
| Sentient, Murmur | RAD |
| Infested Deimos | GAS |
| Overguard layer present | Sentient Incision: COR; Reactive Storm: stays IMP |
| Faction not listed (Stalker, Tenno, Neutral, no health type) | Reactive Storm: stays IMP; Sentient Incision: no element (Decision, UNVERIFIED, D-102) |

### 4.11 Element combination algorithm

```text
E-1.8  Element combination (per attack; positions from 4.10.1, innate group from 4.10.2)

  input : C  = combining contributions, each (pos, sub, element p ∈ PE, value > 0), sorted by (pos, sub)
          IG = innate group members (set of PE); violet = Violet shard ELE bonus (0 if none)
  output: seq (ordered, ≤ 4 distinct PE), groups {final type f → G_f}, ê_p for p ∈ PE

  ê_p ← 0 for every p ∈ PE;  seq ← []
  for c in C (in order):
      ê_{c.element} ← ê_{c.element} + c.value
      if c.element ∉ seq: append c.element to seq          # first occurrence fixes the position;
                                                             # a later mod of the same element only adds value
  ê_ELE ← ê_ELE + violet
  for p in [HEA, COL, ELE, TOX]:                             # HCET
      if p ∈ IG and p ∉ seq: append p to seq                 # innate primaries last; an innate primary already
                                                             # in seq was pulled forward by a mod of its element
  groups ← {}
  for k = 0, 2, 4, …  while k < len(seq):
      if k + 1 < len(seq): groups[COMBINE(seq[k], seq[k+1])] ← {seq[k], seq[k+1]}
      else:                groups[seq[k]] ← {seq[k]}         # odd leftover stays a primary

  COMBINE: {HEA,COL}→BLA  {HEA,ELE}→RAD  {HEA,TOX}→GAS  {COL,ELE}→MAG  {COL,TOX}→VIR  {ELE,TOX}→COR
```

Sources: pairing in sequence order, "each new primary combines with the next uncombined primary" (`F:core` §5.2
rules 1-2; `W:Damage` § Modding example: Cold, Toxin, Fire, Electric in slots 1-4 → (Cold + Toxin) and (Electric +
Fire), lines 873-877); first occurrence (`W:Damage` line 193: "the first position that element is placed in
establishes its hierarchy"; Thermite Rounds before Hellfire; `W:Calculating_Bonuses` line 234); pulled forward ("putting Stormbringer on the top left
slot of an Amprex will change the position of its innate Electricity … to first", line 195); innate last and HCET
(4.10.2); combined primaries "no longer deal their component damage or procs" (`F:core` §5.2 rule 8). The algorithm
reproduces all 48 cells of the wiki's HCET priority table (TV-01-23) and the wiki's worked combinations (TV-01-03,
TV-01-08, TV-01-09).

Notes:

- Four primaries give at most two combined elements; more elemental types only come from innate secondaries and
  non-combining sources (`W:Damage` § Possible Combinations lines 211-212).
- Two innate primaries with no mod element between them combine with each other (Tenet Glaxion Cold + Heat valence →
  Blast, TV-01-07); "both behave as innate elements and combine with each other" (rendered Kuva/Tenet pages).
- A mod-made secondary of the same type as an innate secondary (or as a non-combining secondary) is the same damage
  type and adds to it (E-1.9). This is how per-type damage vectors work and matches the wiki's quantization wording
  "90 % modded Radiation combined with 90 % of Smite Infusion … count … as a sum of their final element (180 %
  Radiation)" (`W:Damage/Calculation` line 118); the Tenet Detron case itself is not stated anywhere (`F:core` §5.2,
  UNVERIFIED, OQ-105).

### 4.12 Composition ratios

```text
E-1.9  Composition r_t of attack a (one stance hit, one target layer)

  for t ∈ IPS:                 r_t = phys_t                                           (E-1.6, E-1.7)
  Reactive Storm (Desert Wind, conversion target w from Table 4.10-A; skipped when w = IMP):
                               n_w ← n_w + r_IMP ;  r_IMP ← 0
  for every group f with G_f:  r_f ← r_f + Σ_{p∈G_f} ( B'_p / B' + ê_p )
  for t ∈ SE ∪ {VOI, TAU, TRU, CIN, FIN}:   r_t ← r_t + B'_t / B'                     (innate secondaries etc.)
  for every type t:            r_t ← r_t + n_t                                       (4.10.3 incl. Unairu on IMP)

  E_t = Σ_{p∈G_t} ê_p + n_t    for elemental t          (overview `sum_elem`)
```

- Every primary with `B'_p > 0` is in seq (4.10.2), so its innate base always joins the final type it ends in.
- Reactive Storm: "Only Desert Wind's Impact damage is altered … Melee Impact mods such as Primed Heavy Trauma,
  increase the amount of altered elemental damage, but do not retain an Impact damage type"; "Damage remains Impact
  … while Overguard is present"; the weakness is fixed per faction, "never choosing a Physical Damage type"
  (`W:Reactive_Storm` lines 28-45, 57). The converted amount is the whole physical-step Impact term (Phoenix Talons
  share included, Unairu Impact not): Decision, UNVERIFIED (OQ-116).
- A non-combining primary whose element was combined elsewhere stays a separate type (Venom Dose example: 100 Impact
  + Infected Clip + Venom Dose at 110 % → "100 Impact, 90 Toxin, and 110 Corrosive", `W:Venom_Dose` line 35;
  TV-01-19).
- Negative r_t (physical bonus below −100 %) are kept (EC-01-08).

### 4.13 Quantization

```text
E-1.10 Quantization (ST-3), once per final type t of the attack instance, f64:
         q_t = 0                                         if r_t = 0
         q_t = sign(r_t) · floor(|r_t| · 32 + 0.5) / 32  otherwise
       Σq = Σ_t q_t                                      (comp_sum_q)
       quantized damage of type t before the remaining multipliers:  q_t · B' · (1 + A)
```

Source: `W:Damage/Calculation` § Quantization lines 12-40 ("round to the nearest multiple of 1/32nd of their
attack's base damage"; `Scale = Modded Base Damage / 32`; `x = Total Damage Type Value / Modded Base Damage`;
`Quantized(x) = sign(x) × floor(|x| × 32 + 0.5) / 32`; negative values: "the same operation is applied to the
absolute magnitude before restoring the sign"). Grid: 1/32 since U40, "(Undocumented) Damage Quantization changed
from 1/16 to 1/32" (`W:Damage` patch history line 603; `F:core` §14). Independent confirmation: the CO catalogue
row for Zhuge Prime's arrow (listed 50) gives the "real in-game value 48.4375 for both the base damage and aptitude
damage" (`W:Condition_Overload_(Mechanic)` line 371); 1/32 reproduces 48.4375 = 50 × 31/32 exactly, 1/16 and no
quantization both give 50 (TV-01-04).

What the scale is:

- The scale is the attack's modded base damage per pellet divided by 32: `B'·(1 + A)/32` in HP. In ratio space the
  grid is 1/32 with denominator B'. B' includes the Strength scaling, the progenitor bonus and the Incarnon flat
  (Decision of overview CD-02 and OQ-002, kept).
- Elemental and physical bonuses do not change the scale ("Elemental and Physical bonuses do not affect the scale",
  `W:Damage/Calculation` line 114); elements use the full base and are rounded on the same grid (line 117).
- The base-damage bucket A, the CO term, faction F, `G`, `G_F`, body part, crit and every enemy-side term are
  outside (overview CD-02). "Get Scale → Quantize Physical and Elemental Damage → Add quantized values → Apply all
  other multipliers" (`W:Damage/Calculation` § Summary lines 237-242). The page's UpdateMe conflict (multipliers
  "only multiply final quantized values" vs "multiply both numerator and scale") gives identical numbers for any
  multiplier that scales every type equally (`F:core` §14).
- The CO term is multiplied by the quantized composition (overview E-12: `W_t = q_t·Y`): the Zhuge Prime row
  above shows the aptitude part is quantized identically to the base part.

Does the Incarnon flat change it? Under E-1.3 `B'_t/B' = (Bw_t + V_t)/B_ref` for every t, every term of E-1.6 to
E-1.9 is a ratio to B', and Phoenix Talons weights scale uniformly, so **every r_t and q_t is identical with and
without X**; X only enlarges B' and hence Y (TV-01-14: Furis Σq = 85/32 with X = 0 and X = 28). The choice "X inside
the scale" is observable only if the real split is not proportional (OQ-030) or if elemental bonuses apply to a base
that excludes X (OQ-102).

Further rules:

- Each attack has its own scale; stance hits with different `S^phys_t` and layer-dependent conversions are quantized
  separately (4.1). Merged beams (CD-10) are quantized per beam before merging.
- DoT seeds use the unquantized Y (E-13), extra hits use the quantized parent hit and are not quantized again
  (overview CD-02).
- Ties at exactly `k + 0.5` round away from zero (TV-01-17). The engine evaluates `floor(abs(r) * 32.0 + 0.5)` in f64
  with r_t computed in the order of E-1.9; whether the game rounds in f32 is open (OQ-107).
- Hitscan weapons do not quantize against Object health (Nullifier bubbles, Tornado), projectiles do
  (`W:Damage/Calculation` § Trivia line 245); Object targets are out of scope (EC-01-11).

### 4.14 Base scalar part handed to `04`

```text
E-1.11 Y0 = B' · max(0, 1 + A)
```

`04` builds the overview's E-11 from Y0 (`none`: `Y = Y0·G_F`; `multiplying`: `Y = Y0·(1 + C)·G_F`; `adding`:
`Y = Y0·G_F + ρ_co·B_ref·C`). The clamp at 0 is a Decision for builds whose base-damage bucket is below −100 %
(EC-01-09), UNVERIFIED (OQ-106).

### 4.15 Faction bucket F

```text
E-1.12 F(f) = Σ_{s ∈ active faction sources} value_s · [f ∈ applies_to(s)]       additive; negative riven values allowed
       weapon-side factor at ST-5: (1 + F(f))     (03 applies it again to DoT, 04 to extra hits)
```

Members (Table 4.15-A) and applicability (Table 4.15-B). Faction mods are a "total damage multiplier", "x1.05 per
rank, up to x1.30 for regular mods and x1.55 for Primed ones" (`W:Faction_Damage_Bonus` lines 1-6): bonus value
0.30 / 0.55. A riven faction stat displayed as `×(1 + v)` has bonus v, which may be negative (`F:mods` §8.3:
×0.71-×0.76 on 2+1 rivens).

Table 4.15-A. Members of F.

| Source | Value | Source of membership |
|---|---|---|
| Bane (rifle), Cleanse (shotgun), Expel (secondary), Smite (melee) of Corpus, Grineer, Infested, Orokin, The Murmur, and their Primed variants | 0.30 / 0.55 | `W:Faction_Damage_Bonus` lines 6, 10-139 |
| Sacrificial Steel and Sacrificial Pressure (Sentient part; set bonus scales it, values `04`) | per `04` | `W:Faction_Damage_Bonus` lines 6, 99-100; `W:Sacrificial_Pressure` ("Similar to other Faction Damage Mods") |
| Paracesis innate Sentient bonus (ranks 31-40, up to 0.60) | per `04` | `W:Faction_Damage_Bonus` lines 146-147; `W:Paracesis` line 15 ("stacks additively … with Roar and the Sacrificial Set") |
| Riven faction stats: Grineer, Corpus, Infested; spliced Orokin, Scaldra, Techrot | v (base 0.45 × formula) | `W:Faction_Damage_Bonus` lines 148-150; `F:mods` §8.4 |
| Roar (and subsumed Roar) | 0.50·Str (0.30·Str), value final per T-13 | `F:buffs` §1.1; `W:Universal_Weapon_Bonuses` § Faction Damage Bonus |
| Damage Blessing (True Master's Font) | 0.25 | `W:Universal_Weapon_Bonuses` § Faction Damage Bonus |
| Summoner's Wrath (aura) | per `04` | "additive to Faction Damage Bonuses" (`W:Summoner's_Wrath` line 29); companion and summon damage only, so in this engine only companion robotic weapons (UNVERIFIED for robotic weapons, OQ-114) |

Table 4.15-B. Which source applies to which target faction (overview §2.2 factions).

| Source group | Applies to | Does not apply to |
|---|---|---|
| Grineer mods, riven Grineer | Grineer, Kuva Grineer | Orokin (Corrupted), Narmer, Scaldra |
| Corpus mods, riven Corpus | Corpus, Corpus Amalgam | Orokin (Corrupted), Narmer |
| Infested mods, riven Infested | Infested, Infested Deimos | Techrot, Narmer |
| Orokin mods, riven Orokin (spliced) | Orokin (Corrupted) | Anarchs; Zariman (Decision) |
| Murmur mods | Murmur | — |
| Sacrificial set (Sentient part), Paracesis | Sentient | — |
| riven Scaldra (spliced) | Scaldra | — |
| riven Techrot (spliced) | Techrot | — |
| Roar, Damage Blessing, Summoner's Wrath | every faction | — |

Sources: "Grineer, Corpus, and Infested Faction Mods have no effect on their Corrupted or Narmer counterparts …
Infested Faction Mods have no effect on Techrot … do apply to Kuva Grineer, Corpus Amalgam, and Infested Deimos
enemies, respectively" (`W:Faction_Damage_Bonus` line 8; `F:addmul` §3); "Currently, Anarchs, Narmer, Scaldra,
Techrot, Stalker, and Wild lack any faction mods" (line 6), so no bane mod reaches Scaldra or Anarchs (Scaldra and
Techrot only through spliced riven stats); faction bonuses apply to "all enemy types of the corresponding Faction,
including Bosses" (line 1). Zariman is in neither list: Decision, no bane mod applies (UNVERIFIED, OQ-113). Stalker,
Tenno and Neutral targets only receive the universal sources.

Further rules:

- Riven faction and bane add in one sum (overview OQ-019, Decision additive, UNVERIFIED).
- F is weapon-side: it applies on every layer, overguard included, and to direct and radial instances alike.
- The faction used here is the target's affiliation (`faction` in `05`'s data), not its `FactionDamageOverride`
  (which only selects type modifiers): Decision, UNVERIFIED (OQ-113).
- Not in F: Mecha Empowered and Vauban passive (they behave like faction on DoT but are `G`/`M_dot`, `04`), target
  vulnerabilities (`04`/`05`), riven "Weak Point Critical Chance" (relative crit chance, `02`; D-010).

---

## 5. Worked example

Checked with `python3`. Tenet Glaxion "Normal Attack" (Cold 34 per tick, `Module:Weapons/data/primary`), Heat
valence 60 %, slot 0 Infected Clip (Toxin +0.90), slot 1 Serration (A = 1.65), one Tauforged Violet shard alone
(ELE 0.45 + 0.15 × 1 = 0.60), target Corpus with Primed Bane of Corpus (F = 0.55). Slots are numbered 0-7 as in
section 4.10.1.

1. E-1.1 to E-1.4: Bw_COL = 34; V_HEA = 0.60 × 34 = 20.4; X = 0; B' = B_ref = 54.4.
2. E-1.8: combining contributions: TOX (slot 0). Innate group: HEA (valence), COL (innate), ELE (Violet); HCET order
   HEA, COL, ELE. seq = [TOX, HEA, COL, ELE] → groups GAS = {TOX, HEA}, MAG = {COL, ELE}; ê_TOX = 0.90, ê_ELE =
   0.60. This is the example of `W:Violet_Archon_Shard` line 43 ("Toxin combines with Heat first to give Gas. Then
   Cold … and finally Electric, which combine to make Magnetic").
3. E-1.9: r_GAS = 20.4/54.4 + 0.90 = 1.275; r_MAG = 34/54.4 + 0.60 = 1.225.
4. E-1.10: q_GAS = floor(40.8 + 0.5)/32 = 41/32 = 1.28125; q_MAG = floor(39.2 + 0.5)/32 = 39/32 = 1.21875; Σq = 80/32
   = 2.5 (here equal to Σr).
5. E-1.11: Y0 = 54.4 × 2.65 = 144.16. Per type before faction: GAS 184.705, MAG 175.695 (Σ 360.4).
6. E-1.12: F = 0.55 (Corpus mods apply to Corpus) → GAS 286.29275, MAG 272.32725, Σ 558.62 per tick (non-crit, no
   CO, `G = G_F = 1`, body hit).

The inspiration orders the same build as [TOX (slot 0), ELE (shard slot 23), COL, HEA] and produces Corrosive (1.5)
and Blast (1.0) instead (D-004, D-006).

---

## 6. Edge cases

Every exception and quirk found in the research files that touches this document's scope. "Out of scope" items name
their owner.

| ID | Rule | Source |
|---|---|---|
| EC-01-01 | Quantization grid is 1/32 (since U40, undocumented change); 1/16 values (inspiration, 2018 research) are obsolete. | `W:Damage` line 603; `F:core` §14; TV-01-04 |
| EC-01-02 | Tiny types round to 0 and vanish (Nagantaka Prime Impact r = 0.01 → q = 0); `02` must not give them proc weight (D-024). | `W:Damage/Calculation` lines 120-131; TV-01-03 |
| EC-01-03 | Galatine Impact (2.5 %) quantizes to 1/32 on the 1/32 grid, not to 0: the wiki trivia "quantization of the Impact damage value to 0 … cannot trigger Shattering Impact" predates U40 (0 only on the 1/16 grid). Engine: 1/32 rule. | `W:Damage/Calculation` line 246; TV-01-16; OQ-112 |
| EC-01-04 | Quantization gains or loses damage per type; Σq ≠ Σr (Braton Prime +0.45 %, Zhuge Prime −3.1 %). The engine never renormalizes Σq. | `W:Damage/Calculation` line 47; TV-01-01, TV-01-04 |
| EC-01-05 | A final type fed by several sources (two primaries, mod + non-combining, innate + mod-made) is summed first and quantized once. | `W:Damage/Calculation` line 118 |
| EC-01-06 | Elemental and physical bonuses do not move the scale; A, CO, F, G, G_F, crit, body part and enemy side are outside. | CD-02; section 4.13 |
| EC-01-07 | The CO part of an Adding attack is multiplied by the quantized composition (both parts 48.4375 on Zhuge Prime). | `W:Condition_Overload_(Mechanic)` line 371; TV-01-04 |
| EC-01-08 | A physical bonus below −100 % (3+1 riven curse, −97.2 % … −118.8 %, `F:mods` §8.3) makes r_t negative; q_t keeps the sign (wiki rule). Downstream proposals in OQ-106. Inspiration clamps to 0 (D-103). | `W:Damage/Calculation` lines 26-40; TV-01-18 |
| EC-01-09 | `1 + A ≤ 0` (large negative riven Damage without damage mods): Y0 = 0 (Decision, UNVERIFIED). | E-1.11; OQ-106 |
| EC-01-10 | Exact ties `k + 0.5` round away from zero. | E-1.10; TV-01-17 |
| EC-01-11 | Hitscan does not quantize against Object health; Object targets are out of scope (single enemy target). | `W:Damage/Calculation` line 245; overview §1.2 |
| EC-01-12 | DoT ticks are not quantized (`03`); extra hits are not re-quantized (`04`). | CD-02; `W:Damage/Calculation` line 617 |
| EC-01-13 | Attacks with an empty damage map (WFCD Haalvu ×2, Simulor and Synoid Simulor "Orb Launch", Shadow Clones) have B' = 0: data error, attack excluded. | `DS:wfcd` §2; section 4.2 |
| EC-01-14 | Progenitor: % of the attack's listed total; base damage for A, elements, quantization scale, CO base and Phoenix Talons; primary e_v in the innate group (HCET); MAG/RAD valence are innate secondaries; IMP valence is physical (Impact mods and Phoenix Talons weights apply, even on an elemental-only attack); same type as an innate type adds. | section 4.5; `F:mods` §10 ("Impact valence … scales like a base-damage addition") |
| EC-01-15 | Progenitor falls off linearly 100 % → 0 % over the weapon's falloff range (`06`, using `progenitor_total`). | rendered Kuva/Tenet pages; `F:weapon` §10.2 |
| EC-01-16 | Progenitor on AoE children and slams: applied (Decision, UNVERIFIED). | OQ-111 |
| EC-01-17 | Valence fusion rounding (down to 0.1 %, ≥ 58 % → 60 %) and the U42 uniform distribution: input only (`09`). | `F:core` §3.2; `F:mods` §10 |
| EC-01-18 | Valence weapons deal half finisher damage (bug): out of scope here, owner `04`/`07`. | `F:addmul` §9; `F:melee` §8 |
| EC-01-19 | Two innate primaries (weapon + valence, or + Violet, or Chromatic Blade + valence) go after all mods in HCET order and combine with each other when no mod element intervenes. | `W:Damage` line 191; TV-01-07, TV-01-23 |
| EC-01-20 | A mod of the same element as an innate primary pulls the innate element to the mod's position (Amprex + Stormbringer). | `W:Damage` line 195; TV-01-09 |
| EC-01-21 | A repeated element keeps the position of its first occurrence; all its values add (Thermite Rounds before Hellfire). | `W:Damage` line 193; TV-01-11 |
| EC-01-22 | Riven elements bottom-up: a mod before the riven combines with its last-listed element; a mod after combines with the first-listed one only if the riven has 1 or 3 elements (with nothing unpaired before it); the valence element behaves like a mod after the riven. | `W:Riven_Mods` lines 1527-1530; TV-01-08 |
| EC-01-23 | Innate secondaries (Ogris/Penta Blast, Stug Corrosive, Nukor/Detron Radiation, MAG/RAD valence) never break up and never absorb a primary (Tenet Detron + Electricity valence keeps Electricity separate); a mod-made secondary of the same type adds to them (UNVERIFIED, OQ-105). | `W:Damage` line 205; TV-01-05, TV-01-06 |
| EC-01-24 | Attacks with an innate primary and an innate secondary (Arquebex and Afentis Prime radial, Mandonel charge): the primary is in the innate group, the secondary stays. | `Module:Weapons/data` scan; E-1.8 |
| EC-01-25 | Combined primaries vanish as damage and as proc types; at most two combined elements come from four primaries. | `F:core` §5.2 rule 8; `W:Damage` line 212 |
| EC-01-26 | Exilus slot is position 8 (after slot 7): Decision; no current exilus weapon mod adds an element. | 4.10.1; overview OQ-024; TV-01-10 |
| EC-01-27 | Primary elements from a weapon arcane or an Incarnon perk (none known today) sit after the exilus and before the innate group: Decision. | 4.10.1; OQ-104 |
| EC-01-28 | Violet shard Electricity combines, from the innate group (HCET), merges with an innate Electricity, is pulled forward by an Electricity mod, is additive with Stormbringer, and its per-shard step counts the shard itself. | `W:Violet_Archon_Shard` lines 40-43; `F:buffs` §5; TV-01-07 |
| EC-01-29 | Non-combining sources (augments since U38, decrees and Saya's Visions since 37.0.7, buffing-field elements, Nourish, Valence Formation, Venom Dose, Smite Infusion, Plunder, Melee Exposure, Crucible Overflow, Unairu Impact, Sentient Incision, Reactive Storm, riven spliced and secondary-element mods, `heatAdd`) are added to their own type after combination; a non-combining primary stays separate even when the same element from mods was combined. | 4.10.3; TV-01-19 |
| EC-01-30 | Thermal Transfer: one field gives its own primary; both fields give one Blast equal to the sum; never combines with weapon elements. | `W:Thermal_Transfer` lines 27-28; D-107 |
| EC-01-31 | Electric Shield ELE stacks +0.5 per shield passed (crit part does not stack, `02`), not Strength-scaled, and does not raise Electricity DoT; Conductive Sphere adds ELE and ELE status damage; Sentient Incision does not scale status damage (flags for `03`). | `W:Electric_Shield` lines 16-18; `F:buffs` §1.12; `W:Sentient_Incision` line 46 |
| EC-01-32 | Unairu Initiative Impact is added like an element (total base), also on weapons with no Impact; the in-game UI shows 25 % per order, the real value is +1.0 per order on weapons; Impact mods do not scale it (Decision, OQ-108). | `W:The_Perita_Rebellion` lines 128-131; TV-01-20 |
| EC-01-33 | Physical mods do nothing for a type the attack lacks (Amprex + Fanged Fusillade); Unairu is the only way to add Impact to such an attack. | `F:core` §1 |
| EC-01-34 | Stance physical bonuses multiply with physical mods and differ per stance hit: composition and quantization per hit. | `F:melee` §2.1; E-1.6 |
| EC-01-35 | Phoenix Talons: 0.3·B' (elements included) added to the physical types, weighted by modded physical shares, after physical mods; nothing for attacks without physical damage and for Void; applies to exalted weapons and Necramechs, not abilities. | `W:Focus/Madurai` lines 26-38; TV-01-12, TV-01-13 |
| EC-01-36 | Exalted weapons: three Strength models; Garuda Talons are not Strength-scaled; Strength is the caster's. | section 4.3; TV-01-13 |
| EC-01-37 | Chromatic Blade: Exalted Blade's physical base becomes one primary (energy colour), which combines with modded elements as an innate element; physical mods and Phoenix Talons then do nothing; slams unaffected. | `W:Chromatic_Blade` lines 38-44; TV-01-13 |
| EC-01-38 | Reactive Storm: only Desert Wind's Impact (after Impact mods) converts to the target's fixed weakness element, non-combining; stays Impact while overguard is present or against enemies without a health type; does not follow Sentient adaptation. | `W:Reactive_Storm` lines 28-46; TV-01-21; D-101 |
| EC-01-39 | Sentient Incision (Venato): +1.2 of the weakness element (Table 4.10-A); Corrosive while overguard is present ("adapts to Overguard health"); no element vs unlisted factions (Decision). | `W:Sentient_Incision` lines 28-46; D-102 |
| EC-01-40 | Physical-to-physical conversion mods are Conclave-only (out of scope); the generic conversion hook adds into the target type (D-009). | `W:Comet_Rounds` lines 3, 26 |
| EC-01-41 | Incarnon flat: per pellet; per-perk attack filter, condition and variant value (`04`/`09`); proportional split (D-005); excluded from the Adding-CO base (`04`); ignores falloff (`06`); ×combo but not ×heavy type multiplier on heavies (`07`). | section 4.6; TV-01-14 |
| EC-01-42 | Incarnon "+Y damage on multishot pellets" (Braton Evo II) and Torid Plentiful Mayhem are not base damage (`G`, `04`). | `F:core` §3.1; `F:mods` §9 |
| EC-01-43 | Negative members of A are legal (Anemic Agility, Vile Acceleration, riven curse). | `F:core` §2 |
| EC-01-44 | Melee projectiles use a second weapon-damage bucket (`04`); members of A carry the `universal_weapon_damage` flag. | `F:addmul` §12 |
| EC-01-45 | Primary Compression is in A on hitscan-AoE weapons and a `G_F` factor on projectile AoE. | `F:addmul` §5 |
| EC-01-46 | Heavy-only melee damage (Killing Blow, Melee Assimilation, riven) enters A only on heavy attacks and heavy slams. | `F:melee` §4 |
| EC-01-47 | Faction applicability per Table 4.15-B (Kuva Grineer, Corpus Amalgam, Infested Deimos yes; Corrupted, Narmer no; Infested not Techrot; Murmur mods work; bosses included). | `W:Faction_Damage_Bonus` lines 1, 8; TV-01-22 |
| EC-01-48 | Riven faction stats are entered as a multiplier (×1.5) and stored as bonus v = 0.5; they can be negative. | `F:mods` §8.3 |
| EC-01-49 | Riven faction and bane are additive (UNVERIFIED). | overview OQ-019 |
| EC-01-50 | F applies on every layer and uses the target's affiliation, not its type-modifier override (Decision). | 4.15; OQ-113 |
| EC-01-51 | Faction applies twice on DoT and three times on extra-hit DoT: owners `03`, `04` (membership here). | `F:core` §6; `F:addmul` §3 |
| EC-01-52 | Summoner's Wrath is in F for companion damage only (robotic weapons UNVERIFIED). | `W:Summoner's_Wrath` line 29 |
| EC-01-53 | Conditional element contributions (Proton Snap, Leaded Gas, Biotic Rounds, Melee Exposure, Valence Formation) hold a position and a value only while active (T-02); in `simulated` mode the composition is recomputed when they change. | 4.10.1; D-105 |
| EC-01-54 | Kitgun grip damage is a per-type vector per chamber; zaw base from `07`; their innate elements (Catchmoon Heat, Plague Keewar Viral) are ordinary innate types. | `F:weapon` §11; `DS:wiki` §3 |
| EC-01-55 | Each attack (AoE child, alt-fire, Incarnon Form) has its own innate elements, base and scale; the same mods produce different final elements per attack. | `F:weapon` §10.1 |
| EC-01-56 | Out of scope: one-shot flat additions (Ballistic Battery, Volt passive, Energy Channel; `F:core` §3.3), Gale Kick (jump kick), Beast claw element conversions (companion AI), Railjack turret faction mods, finisher conversion and its "half of elemental and physical" rule (`04`/`07`). | `F:core` §3.3; `W:Gale_Kick`; `F:addmul` §9 |
| EC-01-57 | Inspiration hooks with no game source are not modelled as game behaviour: `damage_mult` (JS L13194, quirk Q1), `physModsInc` (L8630-8640), `flat_change_dmg_by_strength` (L9640-9643), `impactToPuncture` (L9658-9659). | `I:stat` Q1, §3.3; `I:model` §1.5 ("Engine keys read by `getStatValFromMods` but used by no mod") |
| EC-01-58 | A progenitor and an Incarnon flat on the same attack: no current weapon, but E-1.3 defines the split (TV-01-15). | D-005 |

---

## 7. Inspiration differences

Owned rows of the overview register, restated, then new rows `D-1xx` (the `D-01-xx` range of the brief, numbered
per overview §0.3). Toggle column as in overview §5.1.

| ID | Inspiration behaviour | Game behaviour (source) | Engine | Toggle |
|---|---|---|---|---|
| D-001 | Grid 1/16 of the modded base, `Math.round(value/(unit/16))·(unit/16)` (`setQuantize`, JS L9787-9791) | 1/32 since U40 (`W:Damage` line 603; Zhuge Prime 48.4375, TV-01-04) | E-1.10 on 1/32; compat profile uses 1/16 and the inspiration's unit `T·BM·FAC` | Compat |
| D-002 | Inside the rounding: CO (additive or multiplicative), Longbow Sharpshot, physical mods, finisher, Mirage/Eclipse, `multiple`, frozen, Broadhead, `dbl_mult`, Madurai ×1.3, Cyte factor (JS L9389-9416, L9488-9520; `I:stat` §7, Q5) | only the per-type composition is quantized; every uniform multiplier is outside (`W:Damage/Calculation` § Summary) | CD-02, E-1.10; compat reproduces the inspiration's numerator | Compat |
| D-004 | Innate primaries in the attack JSON's key order, progenitor appended last (JS L12912-12917, L12885-12901; `I:stat` Q6) | HCET order after all mods (`W:Damage` line 191; rendered HCET table) | HCET (E-1.8). The inspiration's order differs in 12 of the 48 cells of the wiki table (TV-01-23) | — |
| D-005 | Flat split proportionally to every existing type incl. progenitor, with factor `(T0 + X)/T0` from the listed total T0 without progenitor, so the progenitor is inflated too: B' = (T0 + V)(1 + X/T0) (`addDmgToBase`, JS L13193-13205) | split undocumented (`F:core` §3.1, open question 1) | E-1.3: proportional to `Bw_t + V_t`, Σ = X exactly (TV-01-15: 72 vs the inspiration's 78); UNVERIFIED (OQ-030) | — |
| D-006 | Violet Electricity listed at shard slots 23-27, i.e. after every mod but before the innate elements (JS L8549-8605, L8606-8629, data L5385-5401) | innate group, HCET with innate and valence elements (`W:Violet_Archon_Shard` lines 40-43) | 4.10.2 (TV-01-07: GAS + MAG, inspiration COR + BLA); the per-shard value formula agrees | — |
| D-007 | "Madurai" ×1.3 on physical types inside the per-type maps; off for `POWER_WEAPON` (exalted) (JS L9254-9256, L9495-9498) | +0.3·B' (elements included) added to physical types by modded physical weights after physical mods; applies to exalted weapons (`W:Focus/Madurai` lines 26-38) | E-1.7 (TV-01-12: Dual Ichor 158.6 unquantized vs the inspiration's 144.5) | User (T-19) + Compat |
| D-008 | Faction keys lack Murmur (Bane of the Murmur gives NaN → ×1) and Corpus Amalgam (Corpus mods and rivens do not hit Amalgam); riven faction rows lack spliced Orokin (JS L8480-8502, L9104-9109, L14717-14721; `I:model` Q2, Q3) | Murmur mods work; Corpus/Grineer/Infested sources apply to Amalgam/Kuva/Deimos (`W:Faction_Damage_Bonus` lines 6-8, 148-150) | Table 4.15-B (TV-01-22) | — |
| D-009 | `swapDmgType` renames Impact to Puncture and overwrites an existing Puncture value (JS L7342-7350); no data uses the hook | no PvE conversion mod exists; the Conclave conversion mods are out of scope (`W:Comet_Rounds` line 3) | generic hook adds into the target type (E-1.1b); UNVERIFIED | — |
| D-101 | Reactive Storm: converts the whole base of attack 0 into a user-chosen type (physical types allowed), which then acts as an innate element and combines with mod elements (JS L7374-7396, L14655-14684; `I:feat` §4) | only Desert Wind's Impact, after Impact mods, converts; target is the fixed faction weakness (never physical); a converted primary does not combine; stays Impact while overguard is up or vs no health type (`W:Reactive_Storm` lines 28-46, 57) | E-1.9 (TV-01-21: vs Infested, Primed Heavy Trauma + Fever Strike: 890.625 vs the inspiration's Gas build 476.5625 for 250 base) | — |
| D-102 | Sentient Incision: the weakness element always comes from the enemy faction; unknown factions fall back to Grineer's (Corrosive); overguard ignored (`addWeaknessDmgType`, JS L7469-7497) | Corrosive while overguard is present; no rule for unlisted factions (`W:Sentient_Incision` lines 28-38) | Table 4.10-A: overguard → COR; unlisted factions → no element (UNVERIFIED) | — |
| D-103 | Each per-type value is clamped at 0 before quantization (`0 > n && (n = 0)`, JS L9415) | negative values keep their sign (`W:Damage/Calculation` lines 26-40) | sign kept (E-1.10); downstream handling proposed in OQ-106 | Compat |
| D-104 | Neutralizer (`strengthType == "cyte"`): separate factor `1.25·Str` on every type, inside the rounding (JS L9356-9358, L9513) | multiplier `m(rank)·Str`, additive with base-damage mods (`W:Neutralize` line 19); Lizzie the same (`W:Exalted_Solo` line 17) | model 2 of E-1.1: `A += m·Str − 1` (form UNVERIFIED, OQ-109) | Compat |
| D-105 | Element list quirks: an element listed at top level occupies a pairing position even when its summed value is 0 (`WITH_COND` without `element` while conditions are on); `WITH_COND.element` never enters the list; `addColdS` adds value but never a position, so its value is lost unless Cold is innate (JS L8549-8629; `I:stat` Q17, Q18) | an element exists only while its effect is active | 4.10.1: a contribution holds a position iff its active value > 0 | — |
| D-106 | Custom `base` and weapon `unique.base` are ignored when ≤ 0 (JS L8442-8445; `I:stat` Q19) | — (custom input; game base mods may be negative) | negative custom `base` enters A | — |
| D-107 | Thermal Transfer stored as one Blast bonus `0.75·Str` (data near JS L4951) | Heat field and Cold field 0.75·Str each; both active → Blast equal to the sum (1.50·Str); never combines with weapon elements (`W:Thermal_Transfer` lines 27-28) | 4.10.3 (values `04`) | — |

Agreements worth recording (no register row): the inspiration reads riven rows in reverse order (JS L7279, `I:stat`
Q24), which is the game's bottom-up rule; its mod-made-plus-innate secondary merge (2025-04-28 and 2025-08-14 fixes,
JS L12941-12973) gives the same Kuva Nukor number as E-1.9 (63.84, TV-01-05); Roar is additive with bane mods
(`getFactionMod`, JS L9104-9109) as in the game; elements are a fraction of the base including flat and progenitor
(`getTotalBaseDmg`) as decided here; Nourish, augment elements, Melee Exposure and riven spliced elements are
non-combining (`add*NotCombined`, `add<Secondary>` keys, JS L12974-13067). Features the inspiration lacks entirely
(Electric Shield and other field elements, Valence Formation, Crucible Overflow, Plunder, Unairu Impact, Phoenix
Talons on exalted weapons, Primary Compression in A) are additions, not discrepancies.

---

## 8. Test vectors

Every vector was computed with the reference implementation and `python3` (f64). Mod values are max rank from
`Module:Mods/data` (Serration 1.65, Hornet Strike 2.2, Hellfire HEA 0.9, Cryo Rounds COL 0.9, Infected Clip TOX 0.9,
Stormbringer ELE 0.9, Fever Strike TOX 0.9, Malignant Force TOX 0.6, Thermite Rounds HEA 0.6, Primed Heated Charge
HEA 1.65, Piercing Caliber PUN 1.2, Primed Heavy Trauma IMP 1.65, Rupture IMP 0.9, Jagged Edge SLA 0.9, Primed Bane
0.55). Base damage is per pellet from `Module:Weapons/data`. "q" is given as k/32; "HP" values are `q_t·Y0` (and
`× (1 + F)` where F is given), with `G = G_F = 1`, no CO, non-crit body hit. Expected outputs must match to 1e-9
relative.

| ID | Inputs | Expected outputs | Checks |
|---|---|---|---|
| TV-01-01 | Braton Prime Normal Attack IMP 1.75, PUN 12.25, SLA 21; Infected Clip slot 4 (5th), Stormbringer slot 5 (6th); A = 1.65; F = 0.55 (Primed Bane of Grineer vs Grineer) | seq [TOX, ELE] → COR; r: IMP 0.05, PUN 0.35, SLA 0.60, COR 1.80; q: 2/32, 11/32, 19/32, 58/32, Σq = 90/32; Y0 = 92.75; HP × (1 + F): 8.985156, 49.418359, 85.358984, 260.569531, Σ 404.332031 | overview §3.8 step 3-4 |
| TV-01-02 | IMP 30, PUN 30, SLA 40 (B' = 100); Hellfire; A = 1.65; F = 0.30 | q: 10, 10, 13, HEA 29 (/32), Σq = 62/32; per 100 base 31.25 + 31.25 + 40.625 + 90.625 = 193.75; × 2.65 = 513.4375; × 1.3 = 667.46875 (unquantized 654.55) | `W:Damage/Calculation` lines 43-118; `F:core` §14 |
| TV-01-03 | Nagantaka Prime IMP 1.73, PUN 15.57, SLA 155.7 (wiki values; module values 1.7/15.6/155.7 give the same q); slots: Cryo Rounds, Malignant Force, Hellfire; Piercing Caliber (PUN 1.2); Valence Formation Gas 2.0 (non-combining); A = 0 | seq [COL, TOX, HEA] → VIR + HEA; q: IMP 0, PUN 6/32, SLA 29/32, HEA 29/32, VIR 48/32, GAS 64/32; HP: 0, 32.4375, 156.78125, 156.78125, 259.5, 346; Σ 951.5 | `W:Damage/Calculation` lines 120-232 |
| TV-01-04 | Zhuge Prime Arrow Impact IMP 10, PUN 22.5, SLA 17.5 (B' = 50); no mods; then CO adding with C = 0.8, ρ_co = 1 | q: 6/32, 14/32, 11/32; Σq = 31/32; total 48.4375 (1/16 grid: 50.0); CO part Σq·B_ref·C = 38.75 = 48.4375 × 0.8 | `W:Condition_Overload_(Mechanic)` line 371 |
| TV-01-05 | Kuva Nukor RAD 21; valence HEA v = 0.60; Stormbringer slot 0; A = 1.65 | V_HEA = 12.6, B' = 33.6; seq [ELE, HEA] → RAD (mod-made) + innate RAD; r_RAD = 1.9 (unquantized 63.84 per pellet before A); q_RAD = 61/32; HP 169.7325. Without mods: q HEA 12/32, RAD 20/32 (12.6 and 21) | `I:stat` §4.5 trace (63.84) |
| TV-01-06 | Tenet Detron RAD 26 per pellet; valence ELE v = 0.60; (a) no mods, (b) Hellfire | V_ELE = 15.6, B' = 41.6. (a) seq [ELE]: ELE 12/32, RAD 20/32 (15.6 and 26, separate). (b) seq [HEA, ELE] → RAD: r 1.9, q 61/32, 79.3 (unquantized 79.04) | `W:Damage` line 205; `F:core` §5.2 |
| TV-01-07 | Tenet Glaxion COL 34; valence HEA 0.60; Infected Clip slot 0; Tauforged Violet alone (ELE 0.60) | V_HEA = 20.4, B' = 54.4; seq [TOX, HEA, COL, ELE] → GAS {TOX, HEA}, MAG {COL, ELE}; q GAS 41/32, MAG 39/32; HP 69.7, 66.3. Without Violet: seq [TOX, HEA, COL] → GAS 41/32, COL 20/32. No mods, no Violet: BLA 32/32 = 54.4 | `W:Violet_Archon_Shard` line 43 |
| TV-01-08 | Braton Prime; riven stats displayed ELE +1.00 then COL +0.80. (a) Hellfire slot 0, riven slot 1; (b) riven slot 0, Hellfire slot 1; (c) riven alone | (a) seq [HEA, COL, ELE] → BLA r 1.7 q 54/32, ELE q 32/32; Σq 118/32. (b) seq [COL, ELE, HEA] → MAG r 1.8 q 58/32, HEA q 29/32; Σq 119/32. (c) MAG 58/32. Also: Tenet Detron (RAD 26, ELE valence 0.60) with a 1-element riven [TOX] in slot 0 → seq [TOX, ELE] → COR + innate RAD; 3-element riven [HEA, COL, TOX] slot 0 + Stormbringer slot 1 → seq [TOX, COL, HEA, ELE] → VIR + RAD | `W:Riven_Mods` lines 1527-1530 example |
| TV-01-09 | Amprex ELE 22. (a) Stormbringer s0, Hellfire s1, Cryo Rounds s2; (b) Hellfire s0, Cryo s1; (c) Hellfire s0; (d) innate ELE with Cold s0, Toxin s1, Heat s2 | (a) seq [ELE, HEA, COL] → RAD r 2.8 q 90/32, COL q 29/32, total 81.8125. (b) seq [HEA, COL, ELE] → BLA 58/32, ELE 32/32, total 61.875. (c) RAD 61/32 = 41.9375. (d) seq [COL, TOX, HEA, ELE] → VIR + RAD | `W:Damage` lines 195, 201; `W:Calculating_Bonuses` lines 232-234 |
| TV-01-10 | Braton Prime; Hellfire slot 0, Infected Clip slot 7, a synthetic Cold 0.9 contribution in the exilus slot (position 8) | seq [HEA, TOX, COL] → GAS r 1.8 q 58/32, COL q 29/32, Σq 119/32 (the "exilus first" reading would give BLA + TOX) | Decision 4.10.1 |
| TV-01-11 | Braton Prime; Infected Clip s0, Thermite Rounds s1, Cryo Rounds s2, Hellfire s3 | seq [TOX, HEA, COL]; GAS = 0.9 + 0.6 + 0.9: r 2.4, q 77/32; COL q 29/32; Σq 138/32; total 150.9375 | `W:Damage` line 193 |
| TV-01-12 | Dual Ichor IMP 19, PUN 11, SLA 45, TOX 47 (B' = 122); Phoenix Talons pt = 0.30. (b) plus Jagged Edge (SLA 0.9) | (a) PT: IMP 0.3×19/75, PUN 0.3×11/75, SLA 0.3×45/75; unquantized 28.272, 16.368, 66.96, 47, Σ 158.6; q 7, 4, 18, 12 (/32); total 156.3125. (b) weights 19, 11, 85.5: r 0.205088, 0.118735, 0.922898, 0.385246; q 7, 4, 30, 12; total 202.0625 | `W:Focus/Madurai` lines 28-34 |
| TV-01-13 | Exalted Blade IMP 37.5, PUN 37.5, SLA 175; Str 1.5 (model 1). (a) Phoenix Talons 0.30; (b) Chromatic Blade HEA + Fever Strike (TOX 0.9) + Jagged Edge + Phoenix Talons | (a) B' = 375; r 0.195, 0.195, 0.91; q 6, 6, 29 (/32); total 480.46875. (b) innate HEA 375; seq [TOX, HEA] → GAS r 1.9, q 61/32, total 714.84375; Jagged Edge and Phoenix Talons add nothing | sections 4.3, 4.4 |
| TV-01-14 | Furis Normal Attack IMP 3, PUN 14, SLA 3; Primed Heated Charge (HEA 1.65); Hornet Strike A = 2.2; (a) X = 0, (b) X = 28 | (a) B' = 20; q 5, 22, 5, HEA 53 (/32), Σq 85/32; Y0 = 64; total 170.0. (b) X_t 4.2, 19.6, 4.2; B' = 48, B_ref = 20; same q; Y0 = 153.6; HP 24, 105.6, 24, 254.4; total 408.0 (compat 1/16: 393.6) | E-1.3, E-1.10 |
| TV-01-15 | Synthetic: B0 IMP 10, PUN 30; valence TOX v = 0.5; X = 12 | V_TOX = 20, B_ref = 60; X_t: IMP 2, PUN 6, TOX 4; B'_t 12, 36, 24; B' = 72 (inspiration: 13, 39, 26, total 78) | D-005 |
| TV-01-16 | Galatine IMP 4.55, PUN 4.55, SLA 172.9 (B' = 182) | r 0.025, 0.025, 0.95; q 1/32, 1/32, 30/32; Σq = 1; HP 5.6875, 5.6875, 170.625 (1/16 grid: IMP and PUN 0, total 170.625) | EC-01-03 |
| TV-01-17 | Synthetic tie: IMP 1, SLA 63 (B' = 64) | r_IMP = 1/64 → abs(r)·32 = 0.5 exactly → q 1/32; r_SLA = 63/64 → 31.5 → q 32/32; Σq = 33/32; total 66 | E-1.10 |
| TV-01-18 | Braton Prime Incarnon Form IMP 28, PUN 2.8, SLA 39.2 (B' = 70); riven Impact −1.15 | r_IMP = 0.4 × (−0.15) = −0.06 → q −2/32 (HP −4.375); PUN 0.04 → 1/32; SLA 0.56 → 18/32; Σq 17/32, total 37.1875 (inspiration: IMP clamped to 0) | EC-01-08, D-103 |
| TV-01-19 | 100 Impact base (B' = 100). (a) Infected Clip + Venom Dose COR 1.1; (b) Hellfire + Smite Infusion RAD 1.1; (c) Infected Clip + Nourish VIR 0.975 (0.75 × Str 1.3), A = 1.65; (d) Infected Clip + Electric Shield ELE 0.5 | (a) IMP 32, TOX 29, COR 35 (/32): 100, 90.625, 109.375 (wiki: 100 / 90 / 110 unquantized). (b) IMP 32, HEA 29, RAD 35. (c) IMP 32, TOX 29, VIR 31 (/32); × 265: 265, 240.15625, 256.71875; Σ 761.875 (`F:buffs` §1.5: 761.9). (d) IMP 32, ELE 16, TOX 29 (/32): no Corrosive | `W:Venom_Dose` line 35; `W:Smite_Infusion` line 34 |
| TV-01-20 | Amprex ELE 22; Unairu Initiative 1 order (IMP +1.0); Rupture (IMP 0.9) | r: IMP 1.0, ELE 1.0; q 32/32 each; total 44 (Rupture adds nothing) | EC-01-32 |
| TV-01-21 | Desert Wind Normal Attack IMP 250; Reactive Storm; Primed Heavy Trauma (IMP 1.65); Fever Strike (TOX 0.9). (a) target Infested; (b) Corpus; (c) overguard present | (a) HEA r 2.65 q 85/32 (664.0625), TOX 29/32 (226.5625), total 890.625. (b) MAG 85/32 + TOX 29/32. (c) IMP 85/32 + TOX 29/32. Inspiration-style (whole base → innate Heat, combines): GAS 61/32 = 476.5625 | D-101 |
| TV-01-22 | Faction F (values): Primed Bane of Corpus vs Corpus Amalgam → 0.55; vs Narmer → 0; Primed Bane of Grineer vs Orokin → 0; Primed Smite Infested vs Techrot → 0; Primed Bane of the Murmur vs Murmur → 0.55 (inspiration 0); Primed Bane of Grineer + riven Grineer ×1.5 + Roar 0.3 vs Kuva Grineer → 1.35; riven Corpus ×0.73 vs Corpus → −0.27. Roar example: 100 Impact, Serration, Bane of Grineer 0.30, Roar 0.50 × Str 1.3, Grineer target | F values as listed; Roar example 100 × 2.65 × (1 + 0.65 + 0.30) = 516.75 | `W:Faction_Damage_Bonus` line 8; `F:buffs` §1.1 |
| TV-01-23 | HCET table: for every (bonus element X, modded element Y ≠ X, innate element W) with one mod Y in slot 0 and innate group {X, W} | the final type set of E-1.8 equals the wiki cell in all 48 cases. Rows (bonus + modded → innate Heat / Cold / Electricity / Toxin): H+C → BLA / BLA / BLA+ELE / BLA+TOX; H+E → RAD / RAD+COL / RAD / RAD+TOX; H+T → GAS / GAS+COL / GAS+ELE / GAS; C+H → BLA / BLA / BLA+ELE / BLA+TOX; C+E → RAD+COL / MAG / MAG / MAG+TOX; C+T → GAS+COL / VIR / VIR+ELE / VIR; E+H → RAD / BLA+ELE / RAD / RAD+TOX; E+C → BLA+ELE / MAG / MAG / MAG+TOX; E+T → GAS+ELE / VIR+ELE / COR / COR; T+H → GAS / BLA+TOX / RAD+TOX / GAS; T+C → BLA+TOX / VIR / MAG+TOX / VIR; T+E → RAD+TOX / MAG+TOX / COR / COR | rendered `Tenet_Detron.parsed.html` "HCET Priority Results" |

Compatibility-profile values (D-001, D-002; for `08`'s oracle tests): TV-01-01 Σq = 2.875 (1/16), 413.3171875 after
faction; TV-01-02 Σq = 1.875, 645.9375; TV-01-04 50.0; TV-01-14 (b) 393.6, (a) 164.0.

---

## 9. Open questions

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-101 | How is the Incarnon flat split across types (restates OQ-030)? | proportional to `Bw_t + V_t` | Braton Prime or Furis with the "+X base damage" perk on a Simulacrum target with a known type modifier (e.g. Grineer, Impact ×1.5): per-type damage numbers before and after selecting the perk |
| OQ-102 | Do elemental bonuses use the base including the Incarnon flat (B') or the original base? | B' (inspiration agrees) | Furis Normal Attack with Primed Heated Charge, with and without the +28 perk, non-crit body hit on an unarmored neutral target: Heat part 79.2 × (1 + A) vs 33 × (1 + A) |
| OQ-103 | Position of the exilus slot in the combine order (overview OQ-024) | after slot 7 (position 8) | only testable once an exilus weapon mod with a primary element exists; recheck `Module:Mods/data` after each update |
| OQ-104 | Position of primary elements from arcanes or Incarnon perks | after exilus, before innate | as OQ-103, when such an effect appears |
| OQ-105 | Does a mod-made secondary add to an innate secondary of the same type (Tenet Detron + Hellfire → one Radiation of 79.04 per pellet before quantization)? | yes | Tenet Detron with Electricity valence and Hellfire on a neutral target: one Radiation number per pellet (79.3 after quantization) vs two |
| OQ-106 | Negative per-type values: does the game keep them (wiki sign rule)? How do enemy-side terms and proc weights treat them? What happens when `1 + A ≤ 0`? | keep the sign; Y0 = 0 for `1 + A ≤ 0`. Proposed for `02`: weights from `max(q_t, 0)`; for `05`: negative types are summed without the per-type minimum of 1 and the instance total is floored at 0 | a 3+1 riven with Impact below −100 % on an Impact-heavy weapon: compare hit numbers with and without the riven against the quantized prediction (TV-01-18 pattern) |
| OQ-107 | Is quantization evaluated in f32, and how are near-ties resolved? | f64, `floor(abs(r)·32 + 0.5)` | a composition whose `abs(r)·32` lies within 1e-6 of `k + 0.5` in f64 but on the other side in f32; damage numbers on Simulacrum |
| OQ-108 | Phoenix Talons with stance physical bonuses, with Unairu Impact, and with the base-damage bucket (overview OQ-020); Unairu Impact scaled by Impact mods? | weights `(1 + P_t)` only; not × stance; Unairu not in weights and not × Impact mods; PT × (1 + A) | Dual Ichor on Simulacrum with Phoenix Talons, with and without Serration-type mods and a stance hit with a physical bonus; a weapon without Impact with Unairu and an Impact mod |
| OQ-109 | Exalted model 2 form (Neutralizer, Lizzie): `A += m·Str − 1` or `A += m·Str`? | `m·Str − 1` | Neutralizer at two Strength values with and without Serration: the ratio of the two hits |
| OQ-110 | Sentient Incision: do its primary weakness elements (Heat, Electricity, Toxin) combine with modded elements? What does it add against Stalker, Tenno and neutral targets? | non-combining; nothing | Venato with Sentient Incision and a Cold mod against Infested: look for Blast |
| OQ-111 | Does the progenitor bonus apply to AoE child attacks and to slams? | yes | Kuva Ogris explosion and a Kuva melee slam, with the valence on and off (Elemental Vice swap does not remove it; compare with an unvalenced copy) |
| OQ-112 | Galatine Impact after U40 (1/32 → 1/32 of the base instead of 0): does Shattering Impact now trigger? | 1/32 rule | Galatine with Shattering Impact on an armored target: armor value before and after Impact procs |
| OQ-113 | Bane applicability where no page states it (Orokin sources vs Zariman) and for enemies whose type-modifier override differs from their faction (Corrupted Jackal, Thrax, Void Angels, Wolf of Saturn Six) | no; affiliation | one hit with and without the bane mod against each such enemy |
| OQ-114 | Does Summoner's Wrath apply to companion robotic weapons? | yes (companion damage) | sentinel weapon hit numbers with and without the aura |
| OQ-115 | Riven faction plus bane: additive (overview OQ-019)? | additive | faction riven + Primed Bane vs the matching faction |
| OQ-116 | Reactive Storm: does the Phoenix Talons share of Impact convert, and does Unairu Impact? | PT share converts, Unairu does not | Desert Wind with Reactive Storm, Phoenix Talons and an Unairu order: per-type numbers vs Infested |
| OQ-117 | Buckets of Arcane Rise, Absorb, Provoked and Primary Plated Round | A | one hit with Serration and the buff vs Serration alone and the buff alone (additive gives `1 + a + b`, multiplicative `(1 + a)(1 + b)`) |
| OQ-118 | Does an inactive conditional element hold its combine position? | no | Proton Snap (Toxin) in slot 0 with Cryo Rounds and Hellfire after it, before and after its condition |

Notes for the manager (proposed overview changes, not applied here):

1. Overview OQ-002: the Zhuge Prime catalogue row ("real in-game value is 48.4375 for both the base damage and
   aptitude damage", `W:Condition_Overload_(Mechanic)` line 371) is direct evidence that the CO part is multiplied by
   the quantized composition and that the grid is 1/32; the second half of OQ-002 (Incarnon flat in the scale) is
   unobservable under D-005's proportional split (section 4.13) and reduces to OQ-030 and OQ-102.
2. Overview E-2: for combined elements `B'_t` must be read as the summed innate base of the group (E-1.9); suggest a
   one-line note.
3. Overview OQ-024: the U38 rule (augment, decree and blessing elements are non-combining) settles the position
   question for Nourish, Valence Formation, field elements and augments (they have no position); only the exilus
   slot and hypothetical arcane or Incarnon primaries remain open (OQ-103, OQ-104).
