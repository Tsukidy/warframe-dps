Status: draft (spec)

# Damage-calculation spec, part 09: build rules

This document decides which builds can exist in the game. It defines the slot layout of every equipment class, which
mods, arcanes, shards, rivens, Incarnon perks and progenitor values may go where, the exclusion and duplicate rules,
mod ranks and their values, drain, polarity, Forma, Catalyst and capacity, set-bonus counting and the Helminth
damage-buff restriction. Every rule is written as a checkable predicate with a stable key and the data field it reads.
The same predicates are the optimizer's hard constraints and the UI's validation messages. This document defines **no
pipeline stage** (overview section 3.1) and no effect values: what a legal item does to damage is owned by the bucket
owners and catalogued in `04`.

Related documents: [00-overview.md](00-overview.md) (master; notation, register, toggles),
[01-damage-composition.md](01-damage-composition.md) (element combination order, which reads the slot order defined
here; progenitor arithmetic), [02-crit-status-multishot.md](02-crit-status-multishot.md),
[03-status-effects.md](03-status-effects.md), [04-conditional-bonuses-and-buffs.md](04-conditional-bonuses-and-buffs.md)
(effect values of mods, arcanes, shards, set bonuses, Incarnon perks, Helminth values; the riven value formula E-4.18
and coefficients), [05-enemies.md](05-enemies.md), [06-ranged-firing-model.md](06-ranged-firing-model.md) (trigger
normalisation, kitgun assembly, Synth Charge and Cannonade effects), [07-melee.md](07-melee.md) (stances, zaw
assembly), [08-evaluation-and-outputs.md](08-evaluation-and-outputs.md) (optimizer metrics).

Contents:

- [1. Scope, inputs and outputs](#1-scope-inputs-and-outputs)
- [2. Build model and slot layout](#2-build-model-and-slot-layout)
- [3. Mod compatibility with an item](#3-mod-compatibility-with-an-item)
- [4. Mutual exclusion and duplicates](#4-mutual-exclusion-and-duplicates)
- [5. Ranks, drain and per-rank values](#5-ranks-drain-and-per-rank-values)
- [6. Capacity, polarity and Forma](#6-capacity-polarity-and-forma)
- [7. Exilus slot](#7-exilus-slot)
- [8. Arcane slots](#8-arcane-slots)
- [9. Archon shard slots](#9-archon-shard-slots)
- [10. Helminth damage-buff restriction](#10-helminth-damage-buff-restriction)
- [11. Set-bonus counting](#11-set-bonus-counting)
- [12. Rivens: legality and value ranges](#12-rivens-legality-and-value-ranges)
- [13. Incarnon evolutions](#13-incarnon-evolutions)
- [14. Progenitor (valence) bonus](#14-progenitor-valence-bonus)
- [15. Worked examples](#15-worked-examples)
- [16. Edge cases](#16-edge-cases)
- [17. Inspiration differences](#17-inspiration-differences)
- [18. Open questions](#18-open-questions)
- [19. Test vectors](#19-test-vectors)

---

## 1. Scope, inputs and outputs

### 1.1 Scope (overview section 7.1, row `09`)

Owns: slot layout per equipment class; mod compatibility (type tree, tags); incompatibility groups; duplicate rules;
capacity, drain, polarity, Forma, Catalyst, overlevelled weapons, aura and stance capacity; exilus; arcane slots
(weapon, warframe, arch-gun pair, kitgun and zaw extra slots; duplicates); archon shard slots; Helminth damage-buff
restriction; set-bonus counting (active weapon since U38.5); riven legality (classes, trait counts and signs, element
limit, splices, one per weapon, disposition 1.2, value ranges for validation); Incarnon tiers (one perk per tier,
variant values); valence fusion range; mod rank scaling.

Not: effect values and condition models (`04`), bucket arithmetic (the bucket owners of overview section 3.1a), the
element combination order (`01`, which consumes the slot indices of section 2.1), kitgun and zaw stat assembly (`06`,
`07`), the optimizer's search algorithm (`docs/architecture/`).

Stages: none. Outputs of this document are inputs of ST-1 to ST-7 (which items are present, at which rank, in which
slot, with which counts).

### 1.2 Citation keys

As overview section 0.2, plus:

| Key | Source |
|---|---|
| `M:Mods`, `M:Weapons`, `M:Arcane` | wiki `Module:Mods/data`, `Module:Weapons/data/<slot>`, `Module:Arcane/data`, as JSON in `research/external/cache/poc/out/` (`Mods_data.json`, `Weapons_data_*.json`, `Arcane_data.json`), read 2026-11-23 |
| `PE:Upgrades` | DE Public Export `research/external/cache/public-export/manifests/ExportUpgrades_en.json` (manifest of 2026-10-22, `DS:pe` §1.7) |
| `W:Semi-Pistol_Cannonade`, `W:Semi-Shotgun_Cannonade` | fetched for this document on 2026-11-23 and cached in `research/external/cache/wiki/` |

Line numbers `L<n>` after a `W:` key refer to the cached raw wikitext file.

### 1.3 Inputs

| Input | Symbol | Data field (source) | Notes |
|---|---|---|---|
| Weapon record | W | `M:Weapons`: `Name`, `Slot`, `Class`, `Family`, `Traits`, `CompatibilityTags`, `Trigger` (weapon and per attack), `Magazine`, `MaxRank`, `Polarities`, `ExilusPolarity`, `StancePolarity`, `Attacks[]`, `InternalName`; module of origin (`primary`, `secondary`, `melee`, `archwing`, `companion`, `modular`) | `DS:wiki` §3; join key to PE is `InternalName` |
| Mod record | m | identity `uniqueName`, `polarity`, `baseDrain`, `fusionLimit`, `type`, `isUtility`, `modSet`, `levelStats` from `PE:Upgrades`; `Type`, `CompatibilityTags`, `IncompatibilityTags`, `Incompatible`, `Set` from `M:Mods` (joined on `InternalName`) | `DS:pe` §1.3, `DS:wiki` §4 |
| Set table | `NumUpgradesInSet` | `M:Mods.Sets`; PE `ExportModSet` | `DS:wiki` §4, `DS:pe` §1.3 |
| Arcane record | a | `M:Arcane`: `Type`, `MaxRank`, `InternalName` | `DS:wiki` §5 |
| Riven coefficients b and the value formula | `riven_coeff`, `riven_value` | `04` §8.2-8.3 (E-4.18); PE riven `upgradeEntries` | owned by `04` |
| Riven trait signs, class availability, splice recipes | – | hand-authored from `W:Riven_Mods` L809-1046, L1294-1398 | no data module (`DS:wiki` §4) |
| Incarnon perk table | – | hand-authored from `W:<Weapon>_Incarnon_Genesis` pages and native weapon pages | absent from every data source (`DS:wiki` §3, `DS:wfcd` §2.5, `DS:pe` §1.6) |
| Archon shard bonus table | – | hand-authored from `W:Archon_Shard` L244-400 | absent from every data source (`DS:pe` §1.6) |
| Helminth restriction table | – | hand-authored from `W:Helminth` L259-278 | |
| Listed base damage per type of the weapon's default attack | `base_listed[t]` (B0_t) | `M:Weapons` `Attacks[]` (`01`) | physical share of E-9.57 |
| Caster of each buff (self or ally) | `buff_str` caster | `04` §4.4 | Helminth rule E-9.32 |
| Toggles | – | T-02, T-24, and the proposed T-30 and T-31 (OQ-915) | overview section 6 |

### 1.4 Outputs and symbols added

Outputs: the legal content of every slot, each mod's rank and the effect value its rank implies (E-9.13), the set counts
`set_count[S]` read by `04` §9, the Archon shard list read by `04` §5.9, the valence fraction read by `01`, the
selected Incarnon perks read by `04` §5.12, the capacity and Forma report, and the list of violated rules.

Symbols added (domain prefixes per overview section 0.3):

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `bld_slot` | s | enum | a slot of an item (section 2.1) |
| `mod_rank`, `mod_rank_max` | r, R | integer | rank of a mod or arcane instance; maximum rank (`fusionLimit`, `M:Arcane.MaxRank`) |
| `mod_drain` | d(r) | capacity points | drain at rank r before polarity (negative for aura and stance mods) |
| `pol_slot`, `pol_mod` | π_s, π_m | Polarity | polarity of a slot, of a mod |
| `cap_slot_cost` | c_s | points | capacity charged by slot s after polarity (E-9.14) |
| `cap_bonus` | β | points | capacity added by the stance (weapon) or aura (warframe) slots (E-9.15) |
| `cap_item` | C | points | item capacity (E-9.16) |
| `cap_used` | U | points | `Σ c_s − β` (E-9.17) |
| `forma_count` | F | count | Forma needed to reach the evaluated polarity layout (E-9.19), reported per Forma type |
| `set_count[S]` | N_S | count | pieces of set S that count (E-9.40) |
| `riv_class` | – | enum | `Rifle`, `Shotgun`, `Pistol`, `Kitgun`, `Melee`, `Zaw`, `Archgun`, `Companion`, `none` |
| `riv_role[i]` | ρ_i | ±1 | +1 for a bonus (positive) trait, −1 for a curse (negative) trait |
| `riv_layout` | (P, N) | counts | number of positive and negative traits |
| `val_tenths` | v10 | integer | progenitor value in units of 0.1 % (`progenitor_frac = v10 / 1000`) |
| `inc_choice[t]` | – | perk id or none | selected perk of Incarnon evolution t |
| `shard[k]` | (colour, tau, bonus) | – | Archon shard k of the warframe |
| `shard_cav` | N_cav | count | equipped Crimson, Azure and Violet shards (input of `04` E-4.12) |
| `hel_subsumed` | – | ability id | the subsumed ability of the self warframe, if any |

### 1.5 Rule form, severities and engine use

Every rule below is an equation `E-9.k` with a stable **key** in backticks (used as the i18n message key) and a
**severity**:

| Severity | Meaning |
|---|---|
| `illegal` | the game does not allow it at the baseline of overview section 1.1 (sourced) |
| `illegal_unverified` | treated as `illegal`, but the rule is UNVERIFIED (an open question names the test) |
| `warning` | possible in game but unusual, or the game has undocumented exceptions; never a constraint |

Engine use (Decision):

1. Validation never blocks evaluation. Both evaluators compute any build, legal or not, so that inspiration links and
   free-form inputs (rivens, valence, custom stats) work as in the inspiration. The UI lists every violation.
2. The optimizer only produces builds that violate no `illegal` or `illegal_unverified` rule of the enabled
   constraint groups. Capacity is a constraint group with a mode (E-9.18, proposed toggle T-30).
3. Items the user supplies (a riven, a valence value, locked slots) are never rewritten by the optimizer. If they
   violate a rule, the optimizer still runs and its report repeats the violation.
4. A rule whose input field is missing in the data is evaluated as stated for the missing value and the data-build
   report lists the weapon or mod (`docs/data/`).

---

## 2. Build model and slot layout

### 2.1 Items and slots

A scenario holds one **weapon item** (the weapon of the evaluated attack; for an exalted weapon, the exalted weapon
itself), one **warframe item**, and an optional user-declared list of set mods on the companion and its robotic weapon
(section 11). Slot identifiers are stable; numbers are the serialization order.

| Item | Slot ids | Content |
|---|---|---|
| weapon | `gen0` … `gen7` | general slots in reading order: top row left to right (0-3), bottom row left to right (4-7). This order is the element combination order of `01` §4.10.1 (`W:Mod` L135-141, `W:Calculating_Bonuses` L230) |
| weapon | `exilus` | exilus slot; position 8 in the element order (`01` EC-01-26) |
| weapon | `stance` | stance slot (melee) |
| weapon | `arcane0`, `arcane1` | weapon arcane slots (section 8) |
| weapon | `riven` | not a slot: a riven occupies one of `gen0` … `gen7` |
| weapon | `inc2` … `inc5` | Incarnon evolution choices (section 13) |
| warframe | `fgen0` … `fgen7`, `fexilus`, `aura0`, `aura1` | warframe mods; `aura1` only on Jade |
| warframe | `farcane0`, `farcane1` | warframe arcanes |
| warframe | `shard0` … `shard4` | Archon shards |
| warframe | `subsumed` | Helminth-infused ability (section 10) |

Item-level fields: weapon `rank` (default `MaxRank`), `catalyst` (default true), `capacity_override` (optional
integer, the capacity the user reads in game), polarity per slot (`pol_gen0` … `pol_gen7`, `pol_exilus`,
`pol_stance`), `genesis_installed` (Incarnon; default true when the weapon has an Incarnon line, which also matches
the inspiration listing the Incarnon-form attack, `I:feat` §4.1), `valence` (element, v10), kitgun and zaw parts (`06`, `07`). Warframe
item: `frame` (optional identity), `reactor` (default true), polarities, `frame_complete` (default false, section 6.5).

### 2.2 Equipment class and slot counts

```text
E-9.1  Equipment class of a weapon record W                                  key: data only
  exalted(W) = 'POWER_WEAPON' ∈ W.CompatibilityTags
  pseudo(W)  = 'POWER_WEAPON_LITE' ∈ W.CompatibilityTags                    (pseudo-exalted melee)
  kitgun(W)  = W comes from Module:Weapons/data/modular ∧ W.Slot ∈ {Primary, Secondary}
  zaw(W)     = W is an assembled zaw (07); its record comes from Module:Weapons/data/modular with Slot Melee
  cls(W) = archgun    if W.Slot ∈ {Archgun, Archgun (Atmosphere)}
           archmelee  if W.Slot = Archmelee
           robotic    if W.Slot ∈ {Robotic, Hound}
           beast      if W.Slot = Beast                                     (out of scope, overview 1.3)
           primary | secondary | melee   if W.Slot = Primary | Secondary | Melee
```

Sources: `DS:wiki` §3 (fields, module list); slot values counted in `M:Weapons` (archwing: `Archgun` 20, `Archgun
(Atmosphere)` 21, `Archmelee` 9; companion: `Robotic` 21, `Beast` 17, `Hound` 3); the four pseudo-exalted weapons
(Landslide Fists, Shadow Clones, Shattered Lash, Whipclaw and their Primes) are exactly the records with
`POWER_WEAPON_LITE` (`W:Exalted_Weapon` L74-88). Kitguns are recognised by origin, not by tag: only 6 of the 12 chamber
records carry `MODULAR_GUN` (EC-09-15).

```text
E-9.2  Slot counts                                                           key: slot.count   severity: illegal
  n_gen(W) = 8 for every class                                              (W:Mod L144-146)
  has_exilus(W) = cls ∈ {primary, secondary, melee} ∧ ¬pseudo(W)            (W:Mod L144-147; W:Exilus_Weapon_Adapter L3-5;
                                                                              W:Exalted_Weapon L96)
  has_stance(W) = cls = melee                                               (W:Mod L145)
  stance_fixed(W) = exalted(W) ∧ cls = melee                                (W:Mod L147; W:Exalted_Weapon L94)
  arcane slots: section 8, E-9.30
  warframe: 8 general, 1 exilus, 1 aura (2 for Jade), 2 arcanes, 5 shards, ≤ 1 subsumed ability
                                                                            (W:Mod L143; W:Arcane_Enhancement L22;
                                                                              W:Archon_Shard L181; W:Helminth L251)
  A build is illegal if it fills a slot that does not exist for its item.
```

| Equipment (E-9.1) | General | Exilus | Stance | Arcane slots (family, E-9.30) | Notes |
|---|---|---|---|---|---|
| primary | 8 | 1 | – | 1 (primary) | kitgun: + 1 (kitgun) |
| secondary | 8 | 1 | – | 1 (secondary) | kitgun: + 1 (kitgun) |
| melee | 8 | 1 | 1 | 1 (melee) | zaw: + 1 (zaw) |
| exalted primary or secondary | 8 | 1 | – | 1 (UNVERIFIED, OQ-906) | Catalyst pre-installed (`W:Exalted_Weapon` L10) |
| exalted melee | 8 | 1 | 1, fixed Zenurik stance | 1 (UNVERIFIED, OQ-906) | Garuda Talons are class Claws, not exalted: free stance |
| pseudo-exalted melee | 8 | 0 | 1, fixed | 1 (UNVERIFIED, OQ-906) | no exilus (`W:Exalted_Weapon` L96) |
| archgun | 8 | 0 | – | 2: `arcane0` primary, `arcane1` secondary | U40 Archgun Arcane Adapter (`F:mods` §1, `W:Arcane_Enhancement` L22) |
| archmelee | 8 | 0 | – | 0 | `W:Arcane_Enhancement` L22 |
| robotic | 8 | 0 | – | 0 | `W:Arcane_Enhancement` L22 ("Sentinel weapons cannot equip …") |
| warframe | 8 | 1 | aura 1 (Jade 2) | 2 (warframe) | 5 shards |

Adapters (Exilus Weapon Adapter, Primary/Secondary/Melee/Archgun Arcane Adapter) and gilding (kitgun and zaw arcanes,
`W:Kitgun` L73, L85; `W:Zaw` L412, L575) are assumed installed: a calculator build describes a fully unlocked item
(Decision).

### 2.3 Slot kinds

```text
E-9.3  Slot-kind compatibility                                              key: slot.kind     severity: illegal
  kind(m) = aura    if PE type = AURA
            stance  if PE type = STANCE
            riven   if m is a riven instance (section 12)
            general otherwise;   exilus_ok(m) = PE isUtility = true
  gen0..gen7, fgen0..fgen7 accept kind ∈ {general, riven}  (exilus-capable mods included)
  exilus, fexilus          accept kind = general ∧ exilus_ok(m)
  stance                   accepts kind = stance; if stance_fixed(W) only the weapon's own exalted stance
  aura0, aura1             accept kind = aura
  rivens are never exilus-capable (they have no isUtility flag)
```

Sources: `W:Mod/Compatibility` L286-289 (aura and stance slots exclusive; exilus mods anywhere, only exilus mods in the
exilus slot); `PE:Upgrades` `type` values AURA 36, STANCE 84 and `isUtility` 195 (`DS:pe` §1.3). The exilus flag comes
from PE `isUtility`, not from wiki `IsExilus`, which disagrees on 7 mods (EC-09-10).

### 2.4 Import of inspiration build links

The inspiration's 30-slot panel (`I:model` §1.2, `JS L5449-5452`, link key `m.<slot>`, `JS L9690-9698`) maps as
follows. The import never rejects a link; it validates after mapping.

| Inspiration slot | Our slot |
|---|---|
| 0-7 | `gen0` … `gen7` (same order) |
| 8, 9, 10 | `exilus`, `stance`, `arcane0` |
| 11, 12 | `farcane0`, `farcane1` |
| 13-16 | `fgen0` … `fgen3` |
| 17 | `aura0` |
| 18 | Incarnon evolution I (form; no choice, sets `genesis_installed`) |
| 19-22 | `inc2` … `inc5` (`{mId, pId}` = tier and perk index) |
| 23-27 | `shard0` … `shard4` (each shard entry maps to colour, tau and bonus; `I:model` §4.4) |
| 28, 29 | archgun `arcane0`, `arcane1` |
| `"Riven"` in 0-7 | the riven in that general slot, traits from link key `r` (`JS L9751-9765`) |

---

## 3. Mod compatibility with an item

A mod may go on a weapon only if its PE type matches the item, its wiki `Type` names a node of the weapon's
compatibility tree, its tags allow the weapon, and any special requirement holds.

### 3.1 Compatibility tree

`W:Mod/Compatibility` L4-201 describes the tree slot → class → sub-class → family → specific weapon; a mod's
`Type` names the node it attaches to and the mod is allowed on every weapon below that node. Kitguns inherit the class
of their chamber, zaws the melee class of their strike (L117, L146, L174).

```text
E-9.4  Compatibility tree                                                    key: compat.type   severity: illegal
  pe_type_ok(m, W):  PE type(m) = PRIMARY for cls primary; SECONDARY for secondary; MELEE for melee general and
                     exilus slots (STANCE for the stance slot, E-9.3); ARCH-GUN for archgun; ARCH-MELEE for archmelee;
                     robotic weapons: the type of their class row below
  nodes(W) = chain(W) ∪ {W.Family, W.Name}
  type_ok(m, W) = pe_type_ok(m, W) ∧ norm_type(m.Type) ∈ nodes(W)
```

| W.Slot / W.Class (`M:Weapons`) | chain(W) |
|---|---|
| Primary / Rifle | Primary, Rifle, Assault Rifle |
| Primary / Launcher, Speargun | Primary, Rifle, Assault Rifle |
| Primary / Bow, Crossbow | Primary, Rifle, Bow |
| Primary / Sniper Rifle | Primary, Rifle, Sniper |
| Primary / Shotgun | Primary, Shotgun |
| Primary / Exalted Weapon | Primary + per weapon: Artemis Bow (Prime) → Rifle, Bow; Neutralizer → Rifle, Sniper; Lizzie → Rifle, Assault Rifle (UNVERIFIED, OQ-916) |
| Secondary / Pistol, Dual Pistols, Shotgun Sidearm, Dual Shotguns, Crossbow | Pistol |
| Secondary / Thrown | Pistol, Thrown |
| Secondary / Tome | Pistol, Tome |
| Secondary / Exalted Weapon | Pistol, Exalted Weapon; Noctua also Tome |
| Melee / class c | Melee, melee_type(c); Glaive also Thrown Melee |
| Melee / Exalted Weapon | Melee, Exalted Weapon |
| Archgun, Archgun (Atmosphere) | Archgun (Arquebex: Archgun) |
| Archmelee | Archmelee (Ironbride: Archmelee) |
| Robotic, Hound / class c | the Primary, Secondary or Melee row of c (Rifle, Sniper Rifle, Shotgun, Pistol, Melee, Glaive) |

`melee_type(c)` is the plural form the mod data uses: Dagger → Daggers, Polearm → Polearms, Sword → Swords, Nikana →
Nikanas, Whip → Whips, Hammer → Hammers, Fist → Fists, Glaive → Glaives, Machete → Machetes, Scythe → Scythes,
Staff → Staves, Tonfa → Tonfas, Warfan → Warfans, Rapier → Rapiers; identical names for Dual Swords, Dual Daggers,
Heavy Blade, Heavy Scythe, Sparring, Nunchaku, Gunblade, Two-Handed Nikana, Dual Nikanas, Assault Saw, Bayonet;
`Blade and Whip` → `Blade And Whip`, `Sword and Shield` → `Sword And Shield`; melee **Claws → none**, because the mod
`Type` "Claws" is used by the beast-claw mods (Bite, Maul …, PE type MELEE, path `/Sentinel/Kubrow/`) and melee claw
stances are matched by tag (E-9.3, E-9.5). Sources: the class lists of `M:Weapons` and the 258 `Type` values of
`M:Mods` (counted for this document).

`norm_type` fixes the non-node `Type` strings of `M:Mods` (EC-09-19): `Rifle (No AoE)` → Rifle, `Pistol (No AoE)` →
Pistol (both mods also carry `IncompatibilityTags` AOE), `{Pistol}` → Pistol, `Unique` (Adhesive Blast) → the weapon
names of a hand-authored override list, `Rifle or Pistol` and `Robotic Weapon` (riven stubs) → not a mod type (rivens
use section 12). Exalted chain rows come from `W:Mod/Compatibility` L114-116, L144-145, L199-201.

### 3.2 Compatibility and incompatibility tags

```text
E-9.5  Tags                                                                  key: compat.tags   severity: illegal
  tags(W) = W.CompatibilityTags ∪ derived(W)            (missing CompatibilityTags = ∅, EC-09-11)
  derived(W) ∋ 'SEMI_AUTO'   if every trigger component of every attack of W is Semi-Auto (E-9.6)
  derived(W) ∋ 'MODULAR_GUN' if kitgun(W)
  derived(W) ∋ stance_tag(class(W)) if zaw(W)          (zaw records carry no tags; class from 07 E-7.42)
  tags_ok(m, W) = (m.CompatibilityTags = ∅ ∨ m.CompatibilityTags ∩ tags(W) ≠ ∅)
                ∧ (m.IncompatibilityTags ∩ tags(W) = ∅)
```

Sources: `W:Mod/Compatibility` L256-259 (Sinister Reach needs `BEAM`; Split Flights excluded by `CRPBOW, CROSSBOW,
GRNBOW, POWER_WEAPON, INFBOW, OMICRUS`); `F:mods` §4. Multi-tag `CompatibilityTags` are read as **any of** (Decision,
UNVERIFIED, OQ-913): every multi-tag list in `M:Mods` is either a companion list (`KAVAT_MOD, KUBROW_MOD, …`, one
companion carries one of them) or `SNIPER_AMMO, LAUNCHER_AMMO` on the Sniper Ammo Mutation pair, and no weapon in
`M:Weapons` carries `LAUNCHER_AMMO`; under "all of" these mods could never be equipped. Stance tags are normalised
before comparison (`BAYONET` → `BAYONET_STANCE`, `SCYTHE_STANCE` → `SCYTHES_STANCE`, EC-09-21).

`stance_tag(c)` is the tag the non-zaw melee weapons of class c carry in `M:Weapons` (majority, counted for this document):
Dagger `DAGGERS_STANCE`, Machete `MACHETES_STANCE`, Nikana `NIKANAS_STANCE`, Polearm `POLEARMS_STANCE`, Rapier
`RAPIER_STANCE`, Scythe `SCYTHES_STANCE`, Staff `STAVES_STANCE`, Sword `SWORDS_STANCE`, Heavy Blade
`HEAVY_BLADE_STANCE`, Hammer `HAMMERS_STANCE` (the classes a zaw strike and grip can produce, `07` E-7.42), and for
completeness Dual Swords `DUAL_SWORDS_STANCE`, Two-Handed Nikana `LONG_KATANA_STANCE`, Dual Nikanas
`DUAL_KATANAS_STANCE`, Assault Saw `BLADESAW_STANCE`, Claws `CLAWS_STANCE_STANCE`, Heavy Scythe `HEAVY SCYTHE_STANCE`,
other classes their own `<CLASS>_STANCE`. Dark Split-Sword carries both the Dual Swords and the Heavy Blade tag (two
forms).

### 3.3 Special equip requirements

Two mod families carry requirements that exist only in their description text (`M:Mods.Description`, checked for
every weapon-type mod with a regex for "Requires", "Only compatible", "Only"):

```text
E-9.6  Cannonade (Semi-Rifle, Semi-Shotgun, Semi-Pistol Cannonade)          key: compat.semi     severity: illegal
  components(a) = split(trigger(a), ' / ')   with trigger(a) = a.Trigger if present else W.Trigger (06 §4.2)
  semi_only(W) = ∀ attack a of W (normal, alt-fire, Incarnon form, child attacks):
                   ∀ c ∈ components(a): c ∈ {'Semi-Auto', 'Semi'}
  The mods require 'SEMI_AUTO' (their CompatibilityTags); E-9.5 derives it from semi_only(W).
  Semi-Rifle and Semi-Shotgun Cannonade carry IncompatibilityTags MODULAR_GUN (primary kitguns excluded);
  Semi-Pistol Cannonade does not (secondary kitguns allowed since U37).
  'Duplex' is not Semi-Auto (Decision, UNVERIFIED, OQ-907).

E-9.7  Synth Charge                                                         key: compat.magazine severity: illegal
  synth_ok(W) = W.Magazine ≥ 6 ∨ W.Name ∈ {Kulstar}          (base magazine of the weapon record, never modded)
  plus E-9.5 with its IncompatibilityTags SINGLESHOT, POWER_WEAPON, SENTINEL_MOD
```

Sources: `W:Cannonade_Mods` L1-5 ("all of the listed trigger types must be Semi"; Argonak excluded, Latron allowed);
`W:Semi-Rifle_Cannonade` L28-31 (Incarnon modes, Nagantaka excluded); `W:Semi-Pistol_Cannonade` L30 (Bronco, Lato,
Lex Incarnon allowed), L35-40 (EFV-8 Mars excluded U38.5.10; secondary kitguns fixed U37);
`W:Semi-Shotgun_Cannonade` L29-30 (Strun Incarnon the only Incarnon shotgun); `M:Mods` (tags quoted). The per-attack
`Trigger` field is missing on most Incarnon-form attacks (Strun, Bronco, Lato, Vectis); inheriting the weapon trigger
reproduces every case the three pages list (Latron, Vectis, Strun, Bronco, Lato, Lex allowed; Argonak, Nagantaka,
Laetum, Vasto, Torid, Dual Toxocyst, EFV-8 Mars excluded), checked against `M:Weapons` for this document. Synth
Charge: `M:Mods.Description` and every PE rank text "Requires Magazine 6 or higher"; `W:Synth_Charge` L27 (base
magazine; raising the magazine does not qualify a weapon; lowering it later does not disqualify), L28 (allowed on the
Burst Laser), L29 (not on exalted weapons), L41 (Kulstar bug); U24.0.1 note (then "5 or higher") superseded.

```text
E-9.8  Mod compatibility                                                    key: compat           severity: illegal
  compatible(m, W) = type_ok(m, W) ∧ tags_ok(m, W) ∧ (m ≠ Synth Charge ∨ synth_ok(W))
                     (the Cannonade rule E-9.6 acts through the derived SEMI_AUTO tag inside tags_ok)
  stance mods: cls(W) = melee ∧ kind(m) = stance ∧ m.CompatibilityTags ∩ tags(W) ≠ ∅  (type_ok not required;
               a stance without tags, such as a beast posture, never fits; Garuda Talons accept any claw stance,
               W:Exalted_Weapon L95)
               if stance_fixed(W): the only legal stance is the stance mod whose M:Mods.Type = W.Family (Exalted
               Blade, Primal Fury, Serene Storm, Razorwing, Hysteria, Ravenous Wraith, Blade Storm, Landslide,
               Shattered Lash, Whipclaw)
  warframe mods: PE type ∈ {WARFRAME, AURA}; an augment (M:Mods.Type = a warframe name) only on that warframe
               when the frame identity is set, otherwise allowed (Decision)
```

---

## 4. Mutual exclusion and duplicates

### 4.1 Exclusion graph

"Two mods with the same hidden parent type or [a] parent/child relationship cannot both be equipped on an item at the
same time" (`W:Mod/Compatibility` L261-262). The parent type is in no data source; the only encoding is the
hand-maintained `Incompatible` name list of `M:Mods` (475 of 1550 mods; `F:mods` §5.2, `DS:wiki` §4).

```text
E-9.9  Exclusion graph                                                      key: excl.pair     severity: illegal
  Excl = { {a, b} : b ∈ a.Incompatible ∨ a ∈ b.Incompatible }        (undirected, both directions, no transitive closure)
  legal(item) ⇔ ∀ distinct mods a, b on the item: {a, b} ∉ Excl
```

Measured on `M:Mods` for this document: every target name exists; 3 asymmetric entries (Primed Electrified Barrel →
Electrified Barrel; Primed Streamline → Streamline, Flawed Streamline), none on primary, secondary or melee mods; the
graph has 3 non-transitive triples, none involving a weapon mod, so a closure would change no weapon build. Families
covered by the lists: normal, Flawed, Primed, Umbra, Amalgam, Galvanized and Archon variants of one mod, plus the
oddities of `F:mods` §5.1 (Critical Delay ⟂ Point Strike; Critical Deceleration ⟂ Blunderbuss; Creeping Bullseye ⟂
Pistol Gambit; Berserker Fury ⟂ Fury; Split Flights ⟂ Split Chamber, Galvanized Chamber; Sacrificial Steel ⟂ True
Steel; Sacrificial Pressure ⟂ Pressure Point; Spectral and Higasa Serration ⟂ Serration; Concealed Explosives ⟂
Energizing Shot). Pairs that **do** stack are simply absent from the lists (Charged Chamber + Primed Chamber, Heavy
Caliber + Serration, Spoiled Strike + Pressure Point, Vigilante Armaments + Split Chamber, Augur Pact + Hornet Strike).

### 4.2 Duplicates

```text
E-9.10  Duplicate rules
  D1 key: dup.mod      illegal              a mod identity (PE uniqueName) appears at most once among all mod slots of one
                                            item (general, exilus, stance, aura): a mod shares its own parent type (E-9.9)
  D2 key: dup.arcane   illegal              an arcane identity appears at most once per item (W:Arcane_Enhancement L26)
  D3                   (allowed)            Archon shards may repeat, same colour and same bonus (W:Archon_Shard L181)
  D4 key: dup.riven    illegal_unverified   at most one riven per weapon item (OQ-904)
  D5                   (allowed)            items are independent: the same mod on two items is legal; the same set mod
                                            on the player's weapon and the robotic weapon counts once (E-9.40)
```

D4 has no wiki statement (`F:mods` §1, open question 4). It follows from D1 if every riven of a class shares one
upgrade type (PE riven records are one per class, `/Lotus/Upgrades/Mods/Randomized/Lotus<Class>RandomModRare`,
`DS:pe` §1.4), which is an inference. Identity is the PE `uniqueName`: names repeat (PE has Beginner and Intermediate
copies under the plain name, WFCD title-cases names, `DS:pe` §1.3, `DS:wfcd` §3.1); Flawed mods are the Beginner
records and are distinct identities linked by the `Incompatible` lists.

---

## 5. Ranks, drain and per-rank values

### 5.1 Rank range

```text
E-9.11 Ranks                                                                key: rank.range    severity: illegal
  mods:    r ∈ [0, R], R = PE fusionLimit (3, 5 or 10 for weapon mods; stances 3; auras 5)
  arcanes: r ∈ [0, R], R = M:Arcane.MaxRank (5; 3 for Kitgun, Zaw and Amp Virtuos arcanes)
  rivens:  r ∈ [0, 8]
  default r = R; the optimizer uses R unless T-31 (proposed, OQ-915) allows lower ranks
```

Sources: `F:mods` §3, §11; `W:Arcane_Enhancement` L28-53; `M:Arcane` (`Type`/`MaxRank`: Kitgun 3, Zaw 3, Amp 3 or 5,
all others 5); `W:Riven_Mods` L809 ("rank 8 (max rank)").

### 5.2 Drain

```text
E-9.12 Drain at rank r (before polarity)
  d(r) = baseDrain + r      if baseDrain ≥ 0
  d(r) = baseDrain − r      if baseDrain < 0     (aura and stance mods: a negative drain adds capacity)
```

Sources: `F:mods` §2.2 (Serration 4 → 14, Galvanized Chamber 6 → 16, Condition Overload 10 → 15); `W:Synth_Charge`
rank table (cost 6, 7, 8, 9); stances `baseDrain` −2 with max rank 3 give −5 (`F:mods` §2.1, `W:Stance` L710 "bonus
mod capacity of 5 when maxed"); Steel Charge −4 at rank 5 gives −9 (`W:Orokin_Reactor` L54, "+9 Max Rank Steel
Charge"). Riven drain is not in any source: it is a user input at the riven's rank (OQ-905).

### 5.3 Per-rank effect values

```text
E-9.13 Per-rank value of a stat line with maximum-rank value x_max
  x(r) = x_max · (r + 1) / (R + 1)                         default (computed in f64 from the maximum value)
  faction multipliers "×(1 + b)": the rule applies to b   (Bane ×1.05 … ×1.30; Flawed Bane ×1.025 … ×1.10)
  affine exceptions: x(r) = x_0 + (x_max − x_0)·r / R     for the lines listed in EC-09-25
  max-rank-only lines: present only at r = R               (Primary/Secondary Merciless reload line, "+1 Arcane Revive")
  set bonuses do not depend on rank (section 11)
```

Sources: `F:mods` §3 (Serration, Primed Pressure Point, Condition Overload, Critical Delay, Galvanized Chamber, Bane;
Merciless), `DS:wfcd` §3.4 (derive lower ranks from the maximum, displayed values are rounded). Checked for this
document over every PRIMARY, SECONDARY, MELEE, ARCH-GUN and ARCH-MELEE record of `PE:Upgrades` with `levelStats`:
967 numeric series follow the proportional rule within display rounding, 37 are affine and 45 deviate only by DE's
irregular display rounding (for example Primary Acuity shows 254 where 350·8/11 = 254.5) or are durations and stack
counts. The damage-relevant affine lines are Internal Bleeding, Hemorrhage and Magnetic Welt (`0.10 + 0.05·r`,
EC-09-25). The engine does not round internal values to the displayed decimals (Decision; `F:mods` §3 "compute from
max-rank value"). Regression test: every catalogued value at every rank, formatted to the display precision of its PE
`levelStats` line, MUST equal that line within one display unit.

Who stores what: `04` stores `x_max` and the condition model of every effect; this document defines `x(r)` and the
exception list. Riven values at rank r use E-4.18 of `04` with its own `(r + 1)` factor.

---

## 6. Capacity, polarity and Forma

### 6.1 Polarities and Forma types

Polarities: Madurai (M), Naramon (N), Vazarin (V), Zenurik (Z), Unairu (U), Penjaga (P), Umbra (UMB), Any (ANY), and
none (unpolarized). Data strings: wiki slot polarity `Aura` means ANY (Vinquibus); wiki mod polarity `Universal`,
`None` and PE `AP_UNIVERSAL` mean "no polarity"; PE `AP_ATTACK` M, `AP_TACTIC` N, `AP_DEFENSE` V, `AP_POWER` Z,
`AP_WARD` U, `AP_PRECEPT` P, `AP_UMBRA` UMB, `AP_ANY` ANY (`W:Polarity` L8-18; `F:mods` §2.3; `DS:pe` §1.3). Weapon
mods only use M, N, V, Z, U, P and UMB (Sacrificial Steel and Pressure); unveiled rivens are M, N or V
(`W:Riven_Mods` L40).

| Forma type | Applies | Restrictions | Source |
|---|---|---|---|
| Forma | any standard polarity (M, N, V, Z, U, P) or none, to any slot including pre-polarized and the exilus slot | unlimited count; resets rank (irrelevant) | `W:Polarity` L24-38; `W:Aura` L313-314 |
| Umbra Forma | UMB | only warframes and melee weapons (and Deconstructor); not directly on a warframe's exilus or aura slot | `W:Umbra_Forma` L2-4, L66-69 |
| Omni Forma | ANY on any slot except the stance slot | – | `W:Polarity` L17-18, L53-54 |
| Stance Forma | ANY on the stance slot | melee only | `W:Polarity` L17-18 |
| Aura Forma | ANY on the aura slot | warframe | `W:Aura` L22, L349 |

### 6.2 Drain after polarity

```text
E-9.14 Capacity charged by a general or exilus slot s holding mod m at rank r (d = d(r) ≥ 0)
  c_s = d                       if π_s = none
  c_s = ceil(d / 2)             if π_s = π_m, or π_s = ANY ∧ π_m ≠ UMB                      (matching)
  c_s = floor(1.25·d + 0.5)     otherwise (mismatch)                                          (Decision, UNVERIFIED)
  alternative (not used): ceil(1.25·d)        differs only for d ≡ 1 (mod 4): 1, 5, 9, 13, 17
  a non-Umbra mod in an UMB slot is a mismatch (Decision, UNVERIFIED, OQ-903)
  a riven uses its own polarity and its user-entered drain
```

Sources: `W:Mod` L162-164 (matching "reduces drain by half, rounded up"; non-matching "increases drain by a quarter,
rounded mathematically: drain of 0-1 by 0, 2-5 by 1, 6-9 by 2, 10-13 by 3, 14-16 by 4"); `W:Polarity` L20 ("50 %,
rounding up"; "25 %, rounded up"); `F:mods` §2.3. The Mod page wins because it gives an explicit bracket table
(per-mechanic page with a numeric table beats prose, overview section 1.4); EC-09-01, OQ-901.

### 6.3 Aura and stance capacity bonus

```text
E-9.15 Capacity added by a stance (weapon) or aura (warframe) slot holding mod m at rank r, b = −d(r) > 0
  β_s = 2·b                     if π_s = π_m or π_s = ANY                                     (matching)
  β_s = b                       if π_s = none
  β_s = floor(0.8·b)            otherwise (mismatch)                                          (Decision, UNVERIFIED)
  alternative (not used): floor(0.75·b + 0.5)    differs for b = 1, 2, 6
  β = Σ β_s over the stance slot (weapon) or the aura slots (warframe)
  exalted melee: the fixed exalted stance in its fixed Zenurik slot gives 2·b (since U38.5)
```

Sources: `W:Aura` L20 ("80 % of listed drain, rounded down (… 5 generates 4, 9 generates 7)"); `W:Polarity` L22
("reduce it by 20 %"); `W:Mod` L166-167 ("reduces it by 25 %, rounded mathematically"); `W:Stance` L709-710 (5 maxed,
10 matching); `W:Exalted_Weapon` L205-210 (U38.5.2: exalted stances give capacity, stance polarity cannot be swapped),
L280 (U38.5: Zenurik polarity on exalted stances, +10). Two of three pages say 20 % and the Aura page gives worked
numbers; every maxed stance (b = 5) gets 4 under all three readings (EC-09-02, OQ-902).

### 6.4 Item capacity

```text
E-9.16 Item capacity                                                       key: cap.rank      severity: illegal
  C = capacity_override                                          if the user entered one
  C = rank · (1 + supercharged)                                  otherwise
    rank ∈ [0, MaxRank]; MaxRank = W.MaxRank (30 or 40), warframe 30; default rank = MaxRank
    supercharged = Orokin Catalyst (weapons, incl. archguns, archmelee, robotic weapons) or Orokin Reactor (warframe);
                   exalted weapons: always true
  overlevel: MaxRank = 40 for the 55 records with M:Weapons.MaxRank = 40 (Kuva, Tenet, Coda weapons, Paracesis,
             Thermian RPG, Kuva archguns); rank 30 + 2k needs at least k Forma applied (k ≤ 5)
```

Sources: `W:Mod` L156 (capacity correlates to rank, maximum 30 or 40; Catalyst and Reactor double it);
`W:Orokin_Catalyst` L4, L52; `W:Orokin_Reactor` L49; `W:Weapons` L21-27 (each Forma +2 maximum rank up to 40 after
5; the extra ranks are doubled by the Catalyst, 80 maximum); `W:Exalted_Weapon` L10 (Catalyst pre-installed);
`F:mods` §2.1 (55 records: 24 primary, 14 secondary, 12 melee, 4 archwing, 1 misc). The minimum-capacity rule for
low-rank items is contradictory between pages (EC-09-55, OQ-908); since builds are evaluated at a given rank, the
engine ignores it and offers `capacity_override` instead (Decision).

### 6.5 Capacity predicate

```text
E-9.17 Capacity used
  weapon:   U = Σ_{s ∈ gen0..gen7, exilus} c_s − β_stance
  warframe: U = Σ_{s ∈ fgen0..fgen7, fexilus} c_s − Σ β_aura
E-9.18 Capacity rule                                                   key: cap.exceeded   severity: illegal
  weapon: U ≤ C under the capacity mode (proposed toggle T-30, OQ-915):
    ignore     no check (the inspiration's behaviour; D-092 compatibility profile)
    current    the build's polarity layout, slot positions free (Swap Polarity, E-9.20): ∃ assignment with F = 0
    any_forma  (default) ∃ layout reachable with any number of Forma: F* of E-9.19 exists
    budget(N)  F* ≤ N
  warframe: checked only when frame_complete = true (the tool models only the damage-relevant frame mods; Decision)
```

`any_forma` is the default because it is the game's own limit: even with every slot matched, 8 Galvanized-class mods
of drain 16 need `8 · 8 = 64 > 60` (TV-09-03), so capacity can bind without regard to Forma.

Order of operations and rounding (all integer arithmetic): (1) `d(r)` per mod (E-9.12; a riven's drain is the user's
integer); (2) per slot, `c_s` or `β_s` with the rounding stated in E-9.14 and E-9.15, each slot rounded on its own;
(3) the sums of E-9.17; (4) the comparison `U ≤ C`. No other rounding occurs, and the order of slots does not matter.
Elsewhere in this document: per-rank effect values are f64 and never rounded to the display (E-9.13); riven ranges
are binary32 (E-9.56, overview NP-2 note on riven values); valence values are integers in tenths (E-9.70, E-9.71).

### 6.6 Minimum Forma

```text
E-9.19 Minimum Forma                                                        (report; constraint in modes any_forma, budget)
  Given: general mods (d_i, π_i) i = 1..k ≤ 8; innate general polarities I (multiset of 8, unfilled entries = none);
         exilus mod (d_x, π_x) and innate exilus polarity; stance mod (b, π_st) and innate stance polarity; capacity C.
  A layout assigns the k mods to distinct general slots (positions free, E-9.20) and decides per occupied slot:
    keep:  the slot keeps its innate polarity p, cost c = E-9.14(d_i, π_i, p), 0 Forma
    forma: the slot gets the mod's polarity, cost ceil(d_i/2), 1 Forma (0 if p already matches);
           if π_i = UMB and Umbra Forma is not allowed on the item (6.1): the slot gets none instead, cost d_i, 1 Forma
  The exilus and stance slots have the same two options (stance: β = 2b with forma; Stance Forma only for ANY;
  no option for a stance_fixed weapon, whose fixed Zenurik stance always gives 2b).
  Unoccupied slots keep their innate polarity.
  F* = min F over layouts with U ≤ C; ties broken by the smaller U. No layout ⇒ cap.exceeded in every mode but ignore.
  Algorithm (exact): dynamic programming over the mods with state = remaining count of innate general slots per
  polarity value (at most 2^8 states) and value = min U per Forma count; then the 2 × 2 exilus and stance options.
  Report: F* by Forma type (Forma, Umbra Forma); for an overlevelled weapon at rank > 30 also the lower bound
  ceil((rank − 30)/2) on Forma applied (E-9.16).
E-9.20 Slot positions                                                       (Decision)
  General-slot polarities are treated as a multiset: once a weapon has been polarized it can rearrange its
  polarities with Swap Polarity for free (W:Polarity L36-37); the data lists innate polarities without positions
  (M:Weapons.Polarities). Exilus, stance and aura slots keep their own polarity (no swap; W:Aura L352 for the aura
  slot; UNVERIFIED for exilus and stance, OQ-911).
```

For a fixed slot assignment, the cheapest way to reach U ≤ C is to Forma the slots with the largest saving
`keep − ceil(d/2)` first, which is how the test vectors were computed by brute force over all assignments (TV-09-02,
TV-09-03). Mod order in the general slots (which drives element combination, `01`) is independent of the polarity
layout under E-9.20.

Vinquibus: the primary and melee records share one Forma layout (`W:Umbra_Forma` L67; EC-09-05). A build of either
form stores the shared layout; the UI shows and edits it once for both forms.

---

## 7. Exilus slot

```text
E-9.21 Exilus                                             keys and severities of E-9.2 (slot.count), E-9.3, E-9.8
  exists: has_exilus(W) (E-9.2); warframe always
  content: E-9.3 (isUtility) ∧ E-9.8
  innate polarity: W.ExilusPolarity (M:Weapons: Madurai or Naramon on primaries and secondaries, None on 249 of 250
                   melee weapons); polarizable with Forma
```

Sources: `W:Exilus_Weapon_Adapter` L1-5 (primary, secondary or melee; melee after Whispers in the Walls, U35); `W:Mod`
L143-147; `W:Exilus_Mods` L442 (U26: exilus slots default to Naramon or Madurai); `F:mods` §1. `W:Exilus_Mods` L5 still
says "primary and secondary weapons" only, which is outdated (EC-09-09).

---

## 8. Arcane slots

```text
E-9.30 Arcane slots and families                                            key: arcane.family severity: illegal
  fam(a) from M:Arcane.Type: Primary, Bow, Shotgun → primary; Secondary (incl. Cascadia, Conjunction, Akimbo)
         → secondary; Melee → melee; Kitgun → kitgun; Zaw → zaw; Warframe → warframe;
         Operator, Amp, Tektolyst Artifacts → not equippable on items of this tool
  slots(W):  primary → arcane0 ∈ primary;  secondary → arcane0 ∈ secondary;  melee → arcane0 ∈ melee
             kitgun(W) → arcane1 ∈ kitgun;   zaw(W) → arcane1 ∈ zaw
             archgun → arcane0 ∈ primary, arcane1 ∈ secondary
             archmelee, robotic → none;  Arquebex, Ironbride (Necramech exalted) → none (Decision)
             exalted weapons → as their slot class (illegal_unverified, OQ-906)
             warframe → farcane0, farcane1 ∈ warframe
  a filled arcane slot that does not exist: slot.count (E-9.2); ranks: E-9.11; duplicates: E-9.10 D2
  Bow, Shotgun and Akimbo arcanes are legal in any slot of their family; their effects are limited to that weapon
  type (04) (Decision, UNVERIFIED, OQ-914)
```

Sources: `W:Arcane_Enhancement` L19-26 (one arcane per weapon; kitgun and zaw arcanes in a separate slot; archguns one
primary and one secondary; sentinel weapons none; archmelee and Necramechs none; no duplicates on one item), L1360-1366,
L1548-1554, L1755-1761 (adapters; archgun slots; separate from Pax and Exodia slots), L1940 (primary passives do not
apply to secondaries); `M:Arcane` type counts; `W:Exalted_Weapon` L121 (U44.0.3 "overlapping Arcane and Exilus Slots
… Exalted Primary or Secondary weapon"), L145-146 (U39.1 arcanes on exalted weapons, exalted melee arcane slot);
`W:Primary_Merciless` L34 (primary exalted weapons with Primary Merciless). Which warframe arcanes act on an archgun
(class-restricted ones such as Arcane Precision do not, `W:Archgun`) is an effect rule of `04` §5.5. The Arcane
Helmet case (one warframe arcane only) is out of scope (EC-09-35).

---

## 9. Archon shard slots

```text
E-9.31 Archon shards                                                                          severity: illegal
  key shard.count: at most 5 shards on the warframe
  key shard.bonus: each shard k: colour_k ∈ {Crimson, Amber, Azure, Topaz, Violet, Emerald}, tau_k ∈ {false, true},
                bonus_k ∈ bonuses(colour_k)                                   (exactly one bonus per shard)
  duplicates: allowed (same colour and same bonus on several shards)
  N_cav = #{k : colour_k ∈ {Crimson, Azure, Violet}}          (input of 04 E-4.12, whatever bonus each shard carries)
```

| Colour | bonuses(colour) (normal value; Tauforged 1.5×) |
|---|---|
| Crimson | melee critical damage; primary status chance; secondary critical chance; Ability Strength; Ability Duration |
| Amber | energy on spawn; health-orb effectiveness; energy-orb effectiveness; casting speed; parkour velocity |
| Azure | health; shield capacity; energy max; armor; health regeneration |
| Topaz | health per Blast kill; shield per Blast kill; secondary critical chance per Heat-affected kill; ability damage vs Radiation-affected |
| Violet | ability damage vs Electricity-affected; primary Electricity (+ per Crimson, Azure, Violet shard); melee critical damage (doubled above 500 max energy); health/energy pickup conversion |
| Emerald | Toxin status damage; health per Toxin tick; ability damage vs Corrosion-affected; Corrosion max stacks |

Sources: `W:Archon_Shard` L181 ("only one can be chosen per Shard (though the same bonus can be chosen again …), and
each Warframe has 5 available Shard slots. Tauforged … 1.5x"), L244-400 (bonus table); `F:buffs` §5. Effects and their
buckets are `04` §5.9. Shards with no weapon effect (Amber, Azure, Crimson Strength) are legal and Azure and Crimson
shards still count in `N_cav` (EC-09-36; D-904).

---

## 10. Helminth damage-buff restriction

Applies only to buffs whose caster is the user's own warframe (`04` §4.4 `buff_str`, caster `self`). Ally-cast buffs
(the default for the toggles T-13 to T-16) are never restricted.

```text
E-9.32 Subsumed abilities                                                   severity: illegal
  H1 key helminth.count: at most one subsumed (Helminth-infused) ability on the warframe item   (W:Helminth L251)
  H2 key helminth.replaced: the subsumed ability replaces one of the frame's four abilities; the replaced native
     ability cannot be a self-cast buff (the user names the replaced ability)
  H3 key helminth.host: if subsumed ∈ R = {Eclipse, Roar, Xata's Whisper} and frame ∈ dom(Host), the replaced
     ability is Host(frame):
       Host = {Chroma: Vex Armor, Cyte-09: Resupply, Mirage: Eclipse, Octavia: Amp, Oraxia: Silken Stride,
               Rhino: Roar, Temple: Ripper's Wail, Uriel: Demonium, Xaku: Xata's Whisper}
     so the self-cast native buff Host(frame) and the subsumed buff are mutually exclusive   (W:Helminth L259-276)
  H4 abilities and frames not listed are unrestricted (Shooting Gallery with Vex Armor is legal) (W:Helminth L278)
  H5 a self-cast subsumed buff uses its Helminth value (04 §5.6)
  H6 Prime and Umbra variants are the same frame for Host (W:Helminth L251 "including those on Prime or Umbra")
```

If the frame identity is not set, H2 and H3 cannot be checked; the engine then only checks H1 and H5 (Decision).

---

## 11. Set-bonus counting

```text
E-9.40 Set count                                                            (no severity: a computed input of 04 §9)
  pieces(S) = { identity(m) : set(m) = S, m ∈ mods(warframe item) ∪ mods(active weapon item) ∪ declared companion
                and robotic-weapon set mods }                                (distinct identities)
  N_S = |pieces(S)|,  0 ≤ N_S ≤ NumUpgradesInSet(S)
  active weapon item = the weapon of the evaluated attack (an exalted weapon when the attack is an exalted attack);
                       set mods on holstered weapons never count (U38.5)
  rank: N_S does not depend on the ranks of the pieces
  set(m) = PE modSet (M:Mods.Set as cross-check); NumUpgradesInSet from M:Mods.Sets
E-9.41 Declared companion pieces                                            key: set.companion  severity: illegal
  a declared piece must be a set mod a companion or robotic weapon can equip; melee robotic weapons (Deconstructor,
  Deconstructor Prime, Akaten, Batoten, Lacerten) cannot equip set mods; Hunter companion mods cannot go on sentinels
```

Sources: `W:Set_Mods` L5-9 (bonus depends on the number of mods equipped, not on fusion), L217-224 (robotic weapons
count; duplicate set mods on the warframe's weapon and the robotic weapon count once; melee robotic weapons cannot
equip set mods; Hunter companion mods not on sentinels), L257-259 (U38.5: active weapon; exalted weapons now count
and benefit), L406 ("Set Mod bonuses require different Mods from the Set"), L229-232 (U41 fixed Gladiator,
Vigilante, Carnis, Mecha, Nira's, Saxum, Synth, Tek bonuses "not applying"); `F:mods` §7.1. Set effects by count
(Vigilante, Gladiator, Sacrificial `modSetValues`, Nira's …) are `04` §9.

---

## 12. Rivens: legality and value ranges

### 12.1 Riven class and binding

```text
E-9.50 Riven class of a weapon
  riv_class(W) = none       if exalted(W) ∨ cls ∈ {archmelee, beast}                (W:Riven_Mods L36, L1526, L1537)
                 Kitgun     if kitgun(W)
                 Zaw        if zaw(W)
                 Archgun    if cls = archgun                         (atmospheric archguns included, L1535-1536)
                 Companion  if cls = robotic                         (veiled robotic rivens fit any robotic weapon, L410)
                 Shotgun    if W.Slot = Primary ∧ W.Class = Shotgun
                 Rifle      if W.Slot = Primary                      (bows, snipers, launchers, crossbows included)
                 Pistol     if W.Slot = Secondary
                 Melee      if W.Slot = Melee
  coefficient column (04 §8.3): Kitgun → Pistol column (also on primary kitguns); Zaw → Melee; Companion → the
                 column of W.Class (Rifle, Sniper Rifle → Rifle; Shotgun → Shotgun; Pistol → Pistol;
                 Melee, Glaive → Melee); others → their own column
E-9.51 Binding                                                              key: riven.binding  severity: illegal
  riv.class = riv_class(W) ≠ none  ∧  riv.family = W.Family            (M:Weapons.Family)
  kitgun: family = chamber name (Catchmoon (Primary) and (Secondary) share 'Catchmoon'); zaw: family = strike name
E-9.52 Count: at most one riven per weapon item (E-9.10 D4)                 key: dup.riven      severity: illegal_unverified
```

Sources: `W:Riven_Mods` L36-44 (classes; one weapon and its variants), L396-414 (veiled class rules: rifle rivens on
non-shotgun primaries, kitgun rivens on primaries or secondaries, robotic on any robotic weapon), L920 (robotic rivens
use the base values of their associated class), L1524-1526 (variants MK1, Prime, Vandal, Wraith, Dex, Prisma, Mara,
Syndicate share; Mutalist Cernos and Quanta, dual vs single sidearms, Dex Dakra vs Dakra Prime, Nikana vs Dragon
Nikana do not; no rivens for exalted weapons); `W:Kitgun` L112-127 (chamber-bound, both versions); `W:Zaw` L642
(strike-bound); `F:mods` §8.2. The wiki `Family` field reproduces every listed pair (Mutalist Cernos, Dex Dakra, Dakra
Prime, Dragon Nikana, Akbolto and Afuris are their own families; Rakta Cernos and Cernos Prime are family Cernos;
Kuva Hek and Vaykor Hek are family Hek), checked on `M:Weapons` for this document; that Kuva, Tenet and Coda variants
share the base weapon's riven is UNVERIFIED (OQ-912).

### 12.2 Traits, layout and signs

A riven instance is `{class, family, rank r ∈ 0..8, polarity ∈ {M, N, V}, drain (user), traits[1..n]}`; traits are in
display order (top to bottom, which `01` reads for the element order) and each is `(trait id, displayed value)`.

```text
E-9.53 Displayed value → v                                                  (input conversion)
  percentage traits:                          v = disp / 100
  Ammo Efficiency, Chance to Gain Combo Count: v = −disp / 100
  faction traits (displayed ×m):              v = m − 1
  Punch Through (m), Range (m), Combo Duration (s), Initial Combo: v = disp
  Parry Angle: v = disp / 100 ("a percentage despite not being shown as such", W:Riven_Mods L987)
E-9.54 Roles and layout                                                     key: riven.layout   severity: illegal
  ρ_i = sign(v_i / b_i)            (b_i = coefficient of trait i for the riven's column; b < 0 for Weapon Recoil
                                    and Ammo Efficiency, so a beneficial value has ρ = +1)
  P = #{ρ_i = +1}, N = #{ρ_i = −1};   require P ∈ {2, 3} ∧ N ∈ {0, 1}  (2 to 4 traits)
E-9.55 Trait rules                                                          key: riven.trait    severity: illegal
  T1 trait ids are distinct
  T2 every trait has a coefficient for the riven's column (04 §8.3; "–" = cannot roll)
  T3 positive-only traits have ρ = +1: Heat, Cold, Electricity, Toxin, Punch Through, Initial Combo, Additional Combo
     Count Chance, every spliced trait;  negative-only: Chance to Gain Combo Count has ρ = −1
  T4 Damage / Melee Damage may have ρ = −1 (Decision, UNVERIFIED: footnote "positive only" contradicts the range
     tables and the Update 44.1 patch example; 04 OQ-424)
  T5 at most one spliced trait; the spliced trait must be available for the class (table below)
  T6 primary elements on one riven: at most P ≤ 3 (follows from T3)
E-9.56 Value range                                                          key: riven.range    severity: illegal
  [v_lo, v_hi] = E-4.18 (04) at the riven's rank r with W = 0.9 and W = 1.1, L from (P, N, ρ_i), D = 1.2,
                 evaluated in binary32 exactly as E-4.18
  legal: v_lo − tol ≤ v_i ≤ v_hi + tol, tol = one display unit of the trait (0.001 for percentage traits,
         0.01 for faction ×, 0.1 for m, s and combo counts)                          (Decision: covers rounding and
                                                                                        truncation of the display, OQ-918)
E-9.57 Physical traits                                                      key: riven.physical severity: warning
  Impact, Puncture or Slash trait with share_t(W) ≤ 0.25, share_t = B0_t / Σ_u B0_u of the weapon's default attack
  (`_TooltipAttackDisplay`, else the first attack)                                                (OQ-910)
E-9.58 Disposition: D = 1.2 for every weapon; M:Weapons.Disposition is a consistency check only (all 1.2),
       `_RetiredDisposition` and WFCD `disposition` are ignored
```

Sources: `W:Riven_Mods` L1-9 (disposition retired, all weapons 1.20×), L40-43 (polarity, MR, one weapon, rerolling),
L776 ("between two and four attributes … Two or three … positive, and one might be negative"), L783-807 (formula and L
table), L809-921 (base values; footnotes 1 positive only, 2 physical > 25 % "usually", 3 negative only), L924-985
(spliced values, always positive), L1294-1310 (only one spliced trait; layout locked; spliced trait inherits the
higher random value, also a negative's; a random trait keeps the trait count, negative if a consumed trait was
negative); `F:mods` §8.1-8.4 (timeline: Hotfix 44.1.3 retirement 2026-10-22; Update 44.1 splicing 2026-10-07; 44.0.2
shotgun Zoom 2026-09-28). A spliced riven keeps the same `(P, N)` layout, so E-9.54 applies unchanged.

### 12.3 Splice recipes (riven generation and validation of T5)

| Spliced trait | Consumed pair(s) | Rifle | Shotgun | Pistol/Kitgun | Archgun | Melee/Zaw |
|---|---|---|---|---|---|---|
| Blast | Heat + Cold | yes | yes | yes | yes | yes |
| Corrosive | Toxin + Electricity | yes | yes | yes | yes | yes |
| Gas | Toxin + Heat | yes | yes | yes | yes | yes |
| Magnetic | Electricity + Cold | yes | yes | yes | yes | yes |
| Radiation | Heat + Electricity | yes | yes | yes | yes | yes |
| Viral | Toxin + Cold | yes | yes | yes | yes | yes |
| Damage vs Orokin | Grineer + Corpus | yes | yes | yes | yes | yes |
| Damage vs Scaldra | Infested + Grineer | yes | yes | yes | yes | yes |
| Damage vs Techrot | Corpus + Infested | yes | yes | yes | yes | yes |
| Status Damage | Damage + Status Chance | yes | yes | yes | yes | yes |
| Weak Point Damage | Damage + Zoom; Damage + Multishot | yes | yes | yes | yes | – |
| Weak Point Critical Chance | Critical Chance + Zoom; Critical Chance + Multishot | yes | yes | yes | yes | – |
| Ammo Efficiency | Magazine + Reload; Recoil + Ammo Maximum | yes | yes | yes | yes | – |
| Reload While Holstered | Ammo Maximum + Reload; Ammo Maximum + Magazine | yes | yes | yes | – | – |
| Melee Damage On Heavy Attack | Heavy Attack Efficiency + Additional Combo Count Chance | – | – | – | – | yes |
| Heavy Attack Wind Up Speed | Heavy Attack Efficiency + Combo Duration | – | – | – | – | yes |
| Parry Angle | Attack Speed + Range | – | – | – | – | yes |
| Slam Attack Damage | Melee Damage + Attack Speed | – | – | – | – | yes |

Source: `W:Riven_Mods` L1312-1398 (recipe table and class availability), L1399-1469 (by weapon type). Coefficients
of spliced traits: `04` §8.3. Companion rivens follow the class column of E-9.50; their vintage traits (Zoom, Recoil,
Ammo Maximum; melee Combo Duration, Slide crit, Finisher, Heavy Attack Efficiency, Initial Combo, Chance to Gain
Combo Count) are legal when present but cannot be cycled in (L921-922), so riven generation (optimizer riven mode)
MUST NOT produce them on companion rivens.

### 12.4 Other riven fields

Rank: 0..8 (E-9.11). Polarity: M, N or V (`W:Riven_Mods` L40). Drain at the current rank: user input with no
validation (no source, OQ-905). MR 8-16: irrelevant. The element order of a riven's traits (bottom-up) is `01`'s.

---

## 13. Incarnon evolutions

```text
E-9.60 Incarnon availability and choices                                                      severity: illegal
  line(W) = the hand-authored perk table row for the exact weapon record W (native Incarnon weapons and every
            variant that accepts its family's Genesis); undefined for Akimbo and Dual variants
  key incarnon.line: a perk or genesis_installed on a weapon without line(W)
  T(W) = 5 evolutions (native: Felarx, Innodem, Laetum, Phenmor, Praedos, Onos, Ruvox, Thalys) or 4 (Genesis)
  evolution I: the Incarnon form (no choice); available iff line(W) defined ∧ (native ∨ genesis_installed)
  key incarnon.tier: evolution t = 2..T(W): inc_choice[t] ∈ perks(W, t) ∪ {none};  |inc_choice[t]| ≤ 1;
                    no choice for t > T(W)
  key incarnon.form: a perk choice requires evolution I to be available
  value(perk, W) = the perk's value in W's variant column (Furis X = 28, Mk1-Furis X = 34)
  the Incarnon-form attacks of the data (M:Weapons attacks named "Incarnon …") are evaluable only when evolution I
  is available
```

Sources: `W:Incarnon` L9 (all sub-types such as Prime, Wraith, Vandal accept the Genesis; Akimbo and Dual weapons do
not), L180-188 ("Each weapon has a total of 4 or 5 evolutions"; "The remaining Evolutions provide 2 or 3 different
selectable perks … Only one perk can be selected per Evolution, which can be changed freely"), L212-216 (Genesis: 4
evolutions; challenges unlock all variants); `W:Furis_Incarnon_Genesis` (cached under `wiki-modules/`) evolution
table: EVO2 Haven Foray, Stormburst (X = 28 / 34); EVO3 Practiced Grip, Extended Volley, Executioner's Fortune; EVO4
Headcracker, Prelude of Might, Elemental Balance (X = 28 % / 39 %); `F:mods` §9; `I:model` §2.3 (the inspiration
stores tiers 18-22 with 18 = form). Perk effects, their attack filters and conditions are `04` §5.12; the perk table
is hand-authored because no data source has it (`DS:wiki` §3, `DS:wfcd` §2.5, `DS:pe` §1.6). The WFCD `Incarnon` tag
marks only 8 weapons and must not be used to detect Incarnon capability (`DS:wfcd` §2.5).

---

## 14. Progenitor (valence) bonus

```text
E-9.70 Valence legality                                                                       severity: illegal
  adversary(W) = W.Traits ∩ {'Kuva Lich', 'Tenet', 'Technocyte Coda'} ≠ ∅      (53 records; IsLichWeapon misses
                                                                                  Kuva Ghoulsaw, EC-09-51)
  V1 (key valence.weapon) a valence is set only on adversary weapons; an adversary weapon without one uses T-24's
     default (0.60) and the UI shows key valence.unset (severity warning)
  V2 (key valence.element) e_v ∈ {IMP, HEA, COL, ELE, TOX, MAG, RAD}
  V3 (key valence.value) v10 ∈ [250, 579] ∪ {600}                 (displayed 25.0 % … 57.9 %, or 60.0 %)
  progenitor_frac = v10 / 1000                (overview symbol v)
E-9.71 Valence fusion (helper for the UI; integer arithmetic in tenths)
  fuse(a10, b10) = 600 if f ≥ 580 else f,   f = min(600, floor(11 · max(a10, b10) / 10))
  the element of the result is the player's choice of the two (U30.5)
```

Sources: `W:Kuva_Weapons` L96 (seven elements, 25-60 % of base damage), L98 (Elemental Vice at rank 40 switches the
element freely); `W:Tenet_Weapons` L116; `W:Coda_Weapons` L145; `W:Valence_Fusion` L13-21 (×1.1 of the higher bonus,
rounded down to the first decimal, capped at 60 %; ≥ 58 % rounds up to 60 %), L59-85 (fusions needed: 52.8 → 1,
48.0 → 2, 43.7 → 3, 39.8 → 4, 36.2 → 5, 33.0 → 6, 30.0 → 7, 27.3 → 8, 25.0 → 9; reproduced exactly by E-9.71),
L89 (reports of hidden precision), L96-99 (U36 58 % rule), L106-112 (U30.5 element choice); `F:mods` §10;
`F:core` §3.2. The roll distribution (uniform since U42 for Kuva and Tenet, `F:mods` §10) does not affect legality.

---

## 15. Worked examples

All arithmetic was checked with `python3` (throwaway scripts in the session scratchpad); riven ranges use numpy
float32 step by step as E-4.18 requires.

**Example 1, capacity and Forma (TV-09-02).** Braton Prime, rank 30, Catalyst: C = 60. `M:Weapons`: no innate general
polarities, exilus Naramon. Mods at maximum rank, drain by E-9.12 from `M:Mods`: Serration 4 + 10 = 14 (M), Galvanized
Chamber 6 + 10 = 16 (M), Primed Bane of Grineer 4 + 10 = 14 (M), Point Strike 2 + 5 = 7 (M), Vital Sense 2 + 5 = 7
(M), Hellfire 6 + 5 = 11 (N), Infected Clip 4 + 5 = 9 (N), Hunter Munitions 4 + 5 = 9 (M). Unpolarized U = 87 > 60.
Savings per Forma `d − ceil(d/2)`: 8, 7, 7, 3, 3, 5, 4, 4. The three largest give 22 (U = 65 > 60); four give 27
(Galvanized Chamber, Serration, Primed Bane, Hellfire): U = 60 ≤ 60. F* = 4, U = 60. Placing Hellfire in a Madurai
slot instead costs floor(1.25 · 11 + 0.5) = 14 (ceil(13.75) = 14 under the alternative rule too) and U = 68 > 60
(`F:mods` §2.4).

**Example 2, riven validation (TV-09-10).** A Braton riven (class Rifle, family Braton) at rank 8 shows Critical
Chance +220.0 %, Multishot +130.0 %, Damage −95.0 %. Roles: b > 0 for all three, so ρ = (+1, +1, −1), layout 2+1,
L = 1.2375 for the positives and −0.495 for the negative. E-4.18 in binary32: Critical Chance b = 0.016666 →
[2.004670, 2.450152]; Multishot b = 0.01 → [1.202850, 1.470150]; Damage b = 0.0183333 → [−1.078108, −0.882088]. All
three values are inside: legal. The qualities are u = 0.4385, 0.3634, 0.3465. A Multishot of +150.0 % would give
u = 1.1117 > 1 and violate `riven.range`.

**Example 3, set count (TV-09-09).** Warframe: Vigilante Pursuit (exilus), Vigilante Vigor. Primary weapon: Vigilante
Armaments, Offense, Fervor, Supplies (exilus). Evaluated attack on the primary: N_Vigilante = 6 (04: tier-raise
chance 0.30). The robotic weapon also carries Vigilante Armaments: still 6 (same identity). Evaluated attack on the
melee weapon: the primary is holstered, N_Vigilante = 2.

---

## 16. Edge cases

| ID | Rule | Source |
|---|---|---|
| EC-09-01 | Non-matching drain: `floor(1.25d + 0.5)` (Mod page) vs `ceil(1.25d)` (Polarity page); they differ for d = 1, 5, 9, 13, 17 (Split Chamber 9 → 11 vs 12). Engine: Mod page; UNVERIFIED. | E-9.14; `W:Mod` L164; `W:Polarity` L20; OQ-901 |
| EC-09-02 | Aura/stance mismatch bonus: 80 % floor (Aura), 75 % rounded (Mod), −20 % (Polarity); a maxed stance (5) gives 4 under all; differs for b = 1, 2, 6. Engine: floor(0.8b). | E-9.15; OQ-902 |
| EC-09-03 | ANY matches every polarity except Umbra; an Umbra mod in an ANY slot is a mismatch; a non-Umbra mod in an Umbra slot is treated as a mismatch (UNVERIFIED). | `W:Polarity` L17; OQ-903 |
| EC-09-04 | Umbra Forma only on warframes, melee weapons and Deconstructor (Prime); not directly on a warframe exilus or aura slot; Umbra mods elsewhere can never be halved. | `W:Umbra_Forma` L66-69 |
| EC-09-05 | Vinquibus primary and melee forms share one Forma layout; Umbra Forma applied through the melee form does not halve anything on the primary. | `W:Umbra_Forma` L67; section 6.6 |
| EC-09-06 | Overlevelled weapons (Kuva, Tenet, Coda, Paracesis, Thermian RPG, Kuva Ayanga and Grattler): rank 40 after 5 Forma; capacity 40, 80 with Catalyst. | E-9.16; `W:Weapons` L21-27 |
| EC-09-07 | Exalted weapons: Catalyst pre-installed; exalted melee stance fixed in a Zenurik slot, +10 capacity since U38.5, polarity not swappable since U38.5.2; pseudo-exalted melee have no exilus slot. | E-9.2, E-9.15; `W:Exalted_Weapon` L10, L94-96, L205-210, L280 |
| EC-09-08 | Garuda Talons: Madurai stance polarity, any claw stance (data class Claws, not exalted). | `W:Exalted_Weapon` L95 |
| EC-09-09 | Melee exilus slot since U35 (after Whispers in the Walls); `W:Exilus_Mods` L5 still says primary and secondary only (outdated). | E-9.21 |
| EC-09-10 | Exilus flag from PE `isUtility`; wiki `IsExilus` differs on 7 mods: Flawed Ravage flagged by the wiki only (data error); Primed Rifle/Shotgun Ammo Mutation, Flawed Whirlwind and three warframe mods flagged by PE only. | E-9.3; measured on `PE:Upgrades` × `M:Mods` |
| EC-09-11 | `CompatibilityTags` missing on about 72 guns (74 primary and secondary records counted for this document, mostly shotguns, pistols and dual pistols, plus the rifle Higasa) and `Polarities` missing on 74 guns: read as empty; the data report lists them. | E-9.5; `DS:wiki` §3, §11; OQ-917 |
| EC-09-12 | `SEMI_AUTO` is on only 12 weapon records (Latron has none): derived from triggers. | E-9.5, E-9.6 |
| EC-09-13 | Cannonade needs every trigger component Semi-Auto: Argonak ("Auto / Semi"), Nagantaka ("Semi / Mag Burst"), Laetum, Vasto (Incarnon Burst), Torid (Incarnon Held), Dual Toxocyst, EFV-8 Mars excluded; Latron, Vectis, Strun, Bronco, Lato, Lex Incarnon allowed; missing per-attack triggers inherit the weapon trigger. | E-9.6 |
| EC-09-14 | Duplex (Tigris, Sancti Tigris, Tigris Prime, Zylok, Zylok Prime) is not Semi-Auto for Cannonade (Decision). | `W:Trigger_Type` L20; OQ-907 |
| EC-09-15 | Kitguns: primary kitguns cannot take Semi-Rifle/Shotgun Cannonade (MODULAR_GUN); secondary kitguns can (U37); MODULAR_GUN is missing on the Gaze, Rattleguts and Vermisplicer records and is derived for every kitgun. | E-9.5, E-9.6 |
| EC-09-16 | Synth Charge: base magazine ≥ 6 (Kulstar 3 allowed by a bug); not on SINGLESHOT or exalted weapons; raising the magazine of a weapon below 6 does not qualify it; equippable on the Burst Laser. Effect rules are `06` EC-06-48. | E-9.7 |
| EC-09-17 | Split Flights excluded from Lenz, Kuva Bramma, exalted weapons, Proboscis Cernos and Nataruk by tags; it excludes Split Chamber and Galvanized Chamber. | E-9.5, E-9.9 |
| EC-09-18 | Vaykor Hek cannot equip Scattered Justice (`SYNDICATEMODBLOCKED`). | E-9.5; `M:Mods`, `M:Weapons` |
| EC-09-19 | Non-node mod `Type` strings (`Rifle (No AoE)`, `Pistol (No AoE)`, `{Pistol}`, `Unique`, `Rifle or Pistol`, `Robotic Weapon`) are normalised; `Claws` is the beast-claw node. | E-9.4 |
| EC-09-20 | Multi-tag `CompatibilityTags` mean "any of" (Sniper Ammo Mutation would be unusable otherwise). | E-9.5; OQ-913 |
| EC-09-21 | Stance tag inconsistencies: Harrowing Spire `BAYONET` vs weapon `BAYONET_STANCE`; one weapon `SCYTHE_STANCE` vs mods `SCYTHES_STANCE`; `CLAWS_STANCE_STANCE` and `HEAVY SCYTHE_STANCE` are consistent on both sides. | E-9.5 |
| EC-09-22 | `Incompatible` lists: 3 asymmetric entries (Primed Electrified Barrel; Primed Streamline twice); union of both directions; no transitive closure. | E-9.9 |
| EC-09-23 | Stacking counter-examples (Charged + Primed Chamber; Heavy Caliber + Serration; Spoiled Strike + Pressure Point; Vigilante Armaments + Split Chamber; Augur Pact + Hornet Strike) vs non-obvious exclusions (Critical Delay ⟂ Point Strike; Critical Deceleration ⟂ Blunderbuss; Berserker Fury ⟂ Fury; Concealed Explosives ⟂ Energizing Shot). | E-9.9; `W:Mod/Compatibility` L262-266 |
| EC-09-24 | Mod identity is the PE `uniqueName`; Beginner (Flawed) and Intermediate copies share the plain name in PE, Primed (Expert) copies too in some records; only identities present in `M:Mods` are offered. | E-9.10 |
| EC-09-25 | Per-rank exceptions: Internal Bleeding, Hemorrhage, Magnetic Welt `0.10 + 0.05r` (not `0.35(r+1)/6`); other affine series without damage effect (Critical Precision, Clip Delegation, Bursting Mass, Static Discharge, ammo mutations …); PE display rounding is irregular, values come from the maximum. | E-9.13 |
| EC-09-26 | Sacrificial Steel/Pressure Sentient bonus is linear per rank in PE (×1.03 … ×1.33, step 0.03); the wiki rank table that looked non-linear (`F:mods` §3) is superseded. Set-level values are `04`'s (OQ-415 there). | E-9.13; `PE:Upgrades` |
| EC-09-27 | Arcane lines that exist only at maximum rank (Merciless reload, "+1 Arcane Revive"); arcane maximum rank 3 for Kitgun and Zaw arcanes. | E-9.11, E-9.13 |
| EC-09-28 | Faction mods scale the bonus part: Bane ×1.05 … ×1.30, Primed ×1.05 … ×1.55, Flawed ×1.025 (shown 1.03) … ×1.10. | E-9.13 |
| EC-09-29 | Set bonus: distinct pieces only ("require different Mods", `W:Set_Mods` L406), active weapon only since U38.5, exalted weapons count since U38.5, rank irrelevant, one piece already gives the 1-piece bonus. | E-9.40 |
| EC-09-30 | Robotic weapon set mods count; the same mod on the player's weapon and the robotic weapon counts once; melee robotic weapons cannot hold set mods; Hunter companion mods not on sentinels. | E-9.40, E-9.41 |
| EC-09-31 | U41 fixed eight set bonuses; data or tests from before U41 may show broken behaviour. | `W:Set_Mods` L229-232 |
| EC-09-32 | Archguns: 2 arcane slots (primary, secondary) since U40; class-restricted warframe arcanes do not act on archguns (effect, `04`); sentinel weapons, archmelee and Necramechs have no arcane slot. | E-9.30 |
| EC-09-33 | Exalted weapons' arcane slots are only evidenced by patch notes (U39.1, U44.0.3) and the Primary Merciless note. | E-9.30; OQ-906 |
| EC-09-34 | Longbow Sharpshot (Bow), Shotgun Vendetta (Shotgun), Akimbo Slip Shot go in the primary/secondary slot; legal on other weapons of that slot, no effect there (Decision). | E-9.30; OQ-914 |
| EC-09-35 | Arcane Helmets (one warframe arcane alongside) are out of scope: the tool assumes two arcane slots. | `W:Arcane_Enhancement` L22 |
| EC-09-36 | Shards: 5 slots, one bonus each, repeats allowed, Tauforged 1.5×; Azure and Crimson shards with non-weapon bonuses still count for the Violet Electricity bonus. Violet/Topaz/Emerald need the Coalescent segment (irrelevant). | E-9.31; `W:Archon_Shard` L181, L227 |
| EC-09-37 | Helminth: only Eclipse, Roar and Xata's Whisper on the 9 host frames are slot-restricted; one infused ability per configuration; ally buffs unrestricted. | E-9.32 |
| EC-09-38 | Riven dispositions retired in Hotfix 44.1.3 (2026-10-22): D = 1.2 for all; `_RetiredDisposition` and WFCD `disposition` (bucketed from a placeholder) are not used. | E-9.58; `DS:wfcd` §2.3; `DS:pe` §1.2 |
| EC-09-39 | Riven binding by `Family`: Mutalist Cernos and Quanta, dual vs single sidearms, Dex Dakra vs Dakra Prime, Nikana vs Dragon Nikana are separate; Kuva/Tenet/Coda variants share the base family (UNVERIFIED). | E-9.51; OQ-912 |
| EC-09-40 | Kitgun rivens bind to the chamber and fit both the primary and secondary kitgun; they use the Pistol column even on primary kitguns. | E-9.50 |
| EC-09-41 | Zaw rivens bind to the strike and use the Melee column. | E-9.50 |
| EC-09-42 | Companion (robotic) rivens use the associated class column; PE has no companion riven record; vintage companion traits are legal but not generated. | E-9.50; section 12.3 |
| EC-09-43 | No rivens for exalted weapons, archmelee weapons or beast claws. | E-9.50 |
| EC-09-44 | Riven Damage: "positive only" footnote vs negative ranges in the overview tables and a patch example; negative allowed. | E-9.55 T4; `04` OQ-424 |
| EC-09-45 | Physical riven traits need > 25 % of that type "usually"; only a warning (exceptions exist). | E-9.57; OQ-910 |
| EC-09-46 | Splice: at most one spliced trait; always positive; the layout stays (the replacement random trait is negative if a consumed trait was); the spliced trait keeps the larger random value, a negative's included; a spliced trait can coexist with its own components (Viral + Cold + Toxin). | E-9.55 T5; `W:Riven_Mods` L1298-1310 |
| EC-09-47 | Shotgun rivens can roll Zoom since 44.0.2 (needed for the Weak Point splices). | `F:mods` §8.1 |
| EC-09-48 | Riven element order (bottom-up), Acuity multishot lock and the multishot floor are not build rules (`01`, `02`). | out of scope |
| EC-09-49 | Riven ranges scale with `(r + 1)`; a rank-0 Rifle 2+1 Critical Chance is 22.27 % … 27.22 %. | E-9.56 |
| EC-09-50 | Valence: ≥ 58 % snaps to 60 % (U36), so 58.0 … 59.9 % is illegal; fusion keeps the higher value; the player picks the element (U30.5); hidden precision (a 52.7 % weapon reported fusing to 60 %). | E-9.70, E-9.71; OQ-909 |
| EC-09-51 | `IsLichWeapon` is missing on Kuva Ghoulsaw; adversary weapons are recognised by `Traits`. | E-9.70 |
| EC-09-52 | Incarnon: 4 evolutions for Genesis, 5 for native; one perk per evolution, changeable; no Genesis for Akimbo/Dual variants; perk values per variant (Furis 28 vs Mk1-Furis 34; Elemental Balance 28 % vs 39 %). | E-9.60 |
| EC-09-53 | Incarnon perk effects (magazine perks not on the form, Torid Plentiful Mayhem, Braton GunCO bug) are effect rules of `04`/`06`. | out of scope |
| EC-09-54 | Ammo Efficiency cap 0.75 (Hotfix 44.1.3) is an effect clamp of `06`; a riven Ammo Efficiency splice is legal at any in-range value. | out of scope |
| EC-09-55 | Minimum capacity of low-rank items: `15 + 1 per 2 MR (+1 per Legendary Rank)` (Mod page) vs `= MR up to 30` (Polarity page) vs `max(15 + rank/2, MR)` (Weapons page); ignored, `capacity_override` instead. | E-9.16; OQ-908 |
| EC-09-56 | Swap Polarity needs a weapon polarized at least once; for a never-polarized weapon whose optimal layout permutes innate polarities, the report notes that 1 Forma may be needed to unlock Swap Polarity. | E-9.20; `W:Polarity` L36 |
| EC-09-57 | Warframe aura slot polarity cannot be swapped with other slots; Jade has two aura slots. | `W:Aura` L352; `W:Mod` L143 |
| EC-09-58 | Catalyst doubles the overlevel ranks too; it cannot be removed and survives Forma. | `W:Weapons` L27; `W:Orokin_Catalyst` L52, L61 |
| EC-09-59 | Weapon augments and Synth Charge cannot go on exalted weapons (`POWER_WEAPON` incompatibility tags, 62 mods). | E-9.5 |
| EC-09-60 | Dex Pixia (Razorwing) is a `Secondary` record in the data and is treated as a secondary; Regulators count as secondaries for arcanes and shards. | E-9.1; `F:weapon` §14 |
| EC-09-61 | Arquebex and Ironbride take Archgun and Archmelee mods; they get no arcane slots (Necramechs have none). | E-9.4, E-9.30; `W:Mod/Compatibility` L199-201 |
| EC-09-62 | Robotic, hound and archwing weapons have no exilus slot; robotic weapons no stance slot. | E-9.2; `W:Mod` L146 |
| EC-09-63 | The riven faction trait list of the inspiration lacks Orokin (D-408 of `04`). | `04` D-408 |
| EC-09-64 | Galvanized stack rules, Galvanized Shot's 3 stacks / 14 s and Galvanized Aptitude vs Braton Incarnon are condition rules of `04`. | out of scope |
| EC-09-65 | Two innate primary elements on adversary weapons (HCET order) are `01`'s. | out of scope |

---

## 17. Inspiration differences

Register rows of overview section 5.2 owned by this document, restated with the engine behaviour, plus new rows in
this document's range `D-901` … (the overview's `D-n01` form for document 09).

| ID | Topic (owner) | Inspiration behaviour | Game behaviour (source) | Decision | Toggle |
|---|---|---|---|---|---|
| D-092 | Mod capacity, polarity and rank (09) | no drain, capacity, polarity, Forma or Catalyst; every mod at its table (maximum-rank) value (`I:opt` §2, §8; no "capacity"/"polarity" in `JS L15252-22038`) | drain per rank, polarity halving and penalty, capacity by rank and Catalyst, Forma (`W:Mod` L156-167, `W:Polarity`, `F:mods` §2-3) | E-9.11 to E-9.20; capacity mode `any_forma` by default; `ignore` reproduces the inspiration | User (proposed T-30, T-31, OQ-915) + Compat (`ignore`) |
| D-093 | Mod incompatibility and duplicates (09) | duplicate guard `isModAlreadyAdded`: any installed name **containing** the candidate's name blocks it (`JS L6998-7007`), which also blocks unrelated pairs; `isModCompatible` checks only the installed mods' `uncomp` lists (one direction, `JS L6987-6997`); `uncomp` data is asymmetric and names non-existent mods (`I:model` Q18-Q19); `showMod` never blocks a duplicate, so links can carry duplicates (`JS L7267-7272`, `I:feat` quirk 7); compatibility by hand-assigned category tags (`JS L7136-7230`) | same parent type ⇒ exclusive; wiki `Incompatible` lists (`W:Mod/Compatibility` L261-262, `F:mods` §5) | E-9.9 (union of both directions), E-9.10 D1-D2, E-9.4-E-9.8 from wiki data | — |
| D-901 | Riven legality (09) | up to 4 free rows, any stat, any value −9999 … 9999, no layout, sign, class, range or disposition rules; the riven pseudo-mod is offered on every weapon incl. exalted (`JS L7273-7341`, `L14685-14845`; `I:feat` §6) | 2-3 positive and 0-1 negative traits, sign rules, class availability, ranges from E-4.18 with D = 1.2, no rivens on exalted weapons (`W:Riven_Mods`) | validate E-9.50 to E-9.58, flag, evaluate the user's values unchanged | — (validation only; numbers unaffected) |
| D-902 | Riven trait catalogue (04, 09) | one list for guns and one for melee (`JS L14701-14754`): no splices except the five factions (D-408), no Finisher, Slide crit, Additional Combo Count, Chance to Gain Combo Count, heavy-attack, slam, parry or Reload While Holstered traits; WP traits mis-mapped (D-010, D-011) | per-class trait table and 18 splices (`W:Riven_Mods` L809-985, L1312-1398) | full per-class catalogue (section 12; coefficients `04` §8.3) | — |
| D-903 | Set-bonus counting (09) | `getCountSetMods` counts equipped panel slots with the same `set.name` (`JS L8514-8518`; `set` data only on the Sacrificial pair); Vigilante and Gladiator are per-piece stats on each mod; duplicates counted, no companion pieces, no active-weapon rule | distinct pieces on warframe + active weapon (+ companion and robotic weapon, deduplicated) since U38.5 (`W:Set_Mods` L217-224, L257-259, L406) | E-9.40 | — |
| D-904 | Archon shards (09, 04) | 18 fixed shard entries (Crimson CD/SC/CC, Topaz CC, Violet CD/Electricity/ability, Emerald stacks/Toxin), offered by weapon category tags; no Amber, Azure or Crimson Strength/Duration entries (`I:model` §4.4), so Azure shards cannot raise the Violet count | 5 slots, any colour, one of its bonuses each (`W:Archon_Shard` L181, L244-400) | E-9.31 | — |
| D-905 | Valence value (01, 09) | any v in [0.25, 0.60], clamped at the input (`JS L14641-14654`); the optimizer enumerates elements at the page value (`I:opt` §5.1) | 0.1 % steps; ≥ 58 % becomes 60 % (`W:Valence_Fusion` L21, L96-99) | E-9.70: 58.0-59.9 % flagged `illegal`, evaluated as entered | — |
| D-906 | Cannonade gating (09) | `trigger: "Semi"` mods need the weapon's single `trigger` field to be `"Semi"` (`JS L7203-7208`); per-attack triggers, Incarnon forms and kitgun rules are not read | every trigger component of every mode must be Semi-Auto; primary kitguns excluded, secondary kitguns allowed (`W:Cannonade_Mods` L5; `W:Semi-*_Cannonade` notes) | E-9.6 | — |
| D-907 | Kitgun and zaw arcane slot (09) | one weapon-arcane slot (10); no Pax, Residual or Exodia arcanes in the data (`I:model` §1.2; arcane entries `JS L4499-4815`) | separate kitgun (Pax/Residual) and zaw (Exodia) slot (`W:Arcane_Enhancement` L22) | E-9.30 | — |
| D-908 | Warframe layout (09) | 4 warframe-mod slots (13-16), no warframe exilus slot; the optimizer uses only slot 13 (`I:feat` §5.1, `I:opt` §2) | 8 general + exilus + aura (Jade 2) (`W:Mod` L143) | full layout; link slots 13-16 map to `fgen0`-`fgen3` | — |
| D-909 | Offering of auras, warframe mods, shards and arcanes (09) | filtered by weapon category tags (Steel Charge only on melee, `JS L7136-7230`, aura entries `JS L4816-4897`) | any aura or warframe mod can be equipped; effects are conditional (`W:Mod/Compatibility` L286-289) | offer all legal items; effect gating per `04` | — |
| D-910 | Helminth restriction (09) | buffs are free toggles; Roar, Eclipse and Xata's Whisper can all be enabled at once (`I:feat` §8) | self-cast combinations restricted (`W:Helminth` L251, L259-278) | E-9.32 only for buffs marked self-cast; ally buffs (the default) unrestricted | — |
| D-911 | Synth Charge equip rule (06, 09) | equippable on any secondary (tag `secondary`, `JS L2171-2176`); effect applied only when modded shots per magazine `H` exceed 6 (`JS L12480-12481`, `L12519-12521`) | equip needs base magazine ≥ 6 (Kulstar bug); effect still works when the magazine is lowered later (`W:Synth_Charge` L27-29, L41) | E-9.7; effect rule `06` EC-06-48 | — |

The riven faction-trait list (`D-408`), the riven Weak Point traits (`D-010`, `D-011`) and the incarnon perk attack
filter (`I:model` Q22) are owned by `04`.

---

## 18. Open questions

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-901 | Non-matching drain: round half up (Mod page) or round up (Polarity page)? | `floor(1.25d + 0.5)` | Split Chamber at rank 5 (drain 9) in a Naramon slot: arsenal shows 11 or 12 |
| OQ-902 | Aura/stance mismatch bonus: 80 % floor or 75 % rounded? | `floor(0.8b)` | a stance at rank 0 or 1 (bonus 2 or 3) in a mismatched slot: 1 vs 2; an aura with bonus 6 (Corrosive Projection rank 4): 4 vs 5 |
| OQ-903 | Drain of a non-Umbra mod in an Umbra slot; Umbra mod in an ANY slot | mismatch; mismatch | arsenal readout of Pressure Point in an Umbra slot and of Sacrificial Steel in an ANY slot |
| OQ-904 | One riven per weapon: no wiki statement | enforced | in-game attempt to equip a second riven of the same weapon; or a patch note |
| OQ-905 | Riven drain: base value range and growth per rank | user input, unchecked | drain readout of several rivens at ranks 0 and 8 |
| OQ-906 | Arcane slots of exalted weapons (primary, secondary, melee), pseudo-exalted melee, Dex Pixia, Arquebex | as their slot class; Arquebex none | arsenal check of each exalted weapon's slot layout |
| OQ-907 | Can Duplex weapons (Tigris, Zylok) equip Cannonade? Are per-attack trigger fields complete for alt-fires? | no; inherit the weapon trigger | in-game attempt; data audit of compound triggers |
| OQ-908 | Minimum capacity formula and Legendary-Rank capacity (three pages disagree) | ignored; `capacity_override` | capacity readout of a rank-30 and an unranked weapon on an MR 30 + LR n account, with and without Catalyst |
| OQ-909 | Valence hidden precision; does a new 58-59.9 % roll still snap to 60 % at drop time? | display tenths are exact; snap at all times | fuse a weapon shown 52.7 % several times and record 57.9 % vs 60 %; collect fresh drops |
| OQ-910 | Physical riven trait threshold: share of which attack, and the exceptions | share of the default attack, warning only | a list of weapons whose rivens roll a physical type below 25 % |
| OQ-911 | Can Swap Polarity move exilus or stance polarities; can a Forma re-apply the same polarity (overlevel Forma count)? | no swap for special slots; not needed | arsenal check |
| OQ-912 | Do Kuva, Tenet and Coda variants share the base weapon's riven (as `Family` says)? | yes | equip a Hek riven on Kuva Hek |
| OQ-913 | Are multi-tag `CompatibilityTags` "any of"? | yes (data evidence) | equip Sniper Ammo Mutation on a sniper rifle (it has `SNIPER_AMMO`, no weapon has `LAUNCHER_AMMO`) |
| OQ-914 | Can Longbow Sharpshot, Shotgun Vendetta and Akimbo Slip Shot be equipped on other weapons of their slot? | yes, no effect | in-game attempt |
| OQ-915 | Proposed toggles for the manager: T-30 capacity mode (`ignore`, `current`, `any_forma` default, `budget(N)`) with allowed Forma types; T-31 allow lower mod ranks in the optimizer (default off); `frame_complete` for warframe capacity | as stated | manager decision (overview section 6) |
| OQ-916 | Compatibility nodes of exalted primaries without a wiki statement (Lizzie) | Primary, Rifle, Assault Rifle | arsenal mod list of Lizzie |
| OQ-917 | Does a missing `CompatibilityTags` field mean "no tags" (72 guns)? | yes | in-game mod list of Higasa (Rifle without tags): is Rifle Ammo Mutation (needs `ASSAULT_AMMO`) offered? |
| OQ-918 | Riven value display: round or truncate (tolerance size); is W computed in f32 before the product (`04` OQ-424)? | one display unit tolerance; per E-4.18 | riven readouts at the ends of a range (from trait locking and splicing) |

---

## 19. Test vectors

Each vector gives every input needed; data values were read from `M:Mods`, `M:Weapons` and `PE:Upgrades` (cached
2026-11-23). Arithmetic checked with `python3`.

**TV-09-01 Drain and polarity (E-9.12, E-9.14, E-9.15).**

| Input | Expected |
|---|---|
| Serration base 4, r = 10: d | 14; matching slot 7; unpolarized 14; mismatch 18 (alternative rule also 18) |
| Split Chamber base 4, r = 5, Madurai mod in a Naramon slot | d = 9; mismatch 11 (alternative 12) |
| d = 1, 5, 13, 17 mismatch | 1, 6, 16, 21 (alternative 2, 7, 17, 22) |
| d = 0 any slot | 0 |
| Sacrificial Steel (UMB) d = 16 in an ANY slot | mismatch 20 |
| Serration d = 14 in an ANY slot | 7 |
| Blind Justice base −2, r = 3 (b = 5): matching, unpolarized, mismatch | 10, 5, 4 (alternative 4) |
| stance r = 0 (b = 2) mismatch | 1 (alternative 2) |
| Steel Charge base −4, r = 5 (b = 9) | matching 18, mismatch 7 |

**TV-09-02 Capacity and minimum Forma, no innate polarities (E-9.16 to E-9.19).** Braton Prime (`Polarities` nil,
`ExilusPolarity` Naramon, `MaxRank` 30), Catalyst, rank 30, exilus empty. Mods (d, polarity): Serration 14 M,
Galvanized Chamber 16 M, Primed Bane of Grineer 14 M, Point Strike 7 M, Vital Sense 7 M, Hellfire 11 N, Infected Clip
9 N, Hunter Munitions 9 M. Expected: C = 60; unpolarized U = 87; F* = 4, U = 60 (`cap.exceeded` false in
`any_forma` and in `budget(4)`, true in `budget(3)` and `current`). With 3 Forma the best U is 65.

**TV-09-03 Capacity with an innate polarity and an infeasible build.** (a) Lex Prime (`Polarities` [Madurai],
`MaxRank` 30), Catalyst, rank 30. Mods: Hornet Strike 14 M, Galvanized Diffusion 14 M, Pistol Gambit 9 M, Primed
Target Cracker 14 M, Lethal Torrent 11 M, Primed Heated Charge 16 N, Pistol Elementalist 9 V, Synth Charge 9 M.
Expected: unpolarized sum 96; F* = 5, U = 58 (the innate Madurai slot holds a 14-drain Madurai mod; Forma on Primed
Heated Charge, two of the remaining 14s, Lethal Torrent and one 9). Same result under the alternative mismatch rule.
(b) Any weapon with C = 60 and eight Madurai mods of drain 16: all-matched U = 64 > 60, no layout exists,
`cap.exceeded` true in every mode except `ignore`.

**TV-09-04 Stance capacity and exalted capacity (E-9.15, E-9.16).** (a) Nikana Prime (`StancePolarity` Madurai),
Catalyst, rank 30, Blind Justice (M) rank 3 in the stance slot: β = 10, so mods may use U_gen ≤ 70. Same stance in an
unpolarized stance slot: β = 5 (65); in a Vazarin slot: β = 4 (64). (b) Exalted Blade: Catalyst forced, rank 30, fixed
Zenurik stance (b = 5): C = 60, β = 10.

**TV-09-05 Overlevel (E-9.16).** Kuva Hek (`MaxRank` 40): rank 40, Catalyst → C = 80; rank 36 → 72; rank 40 without
Catalyst → 40; Forma lower bound for rank 40: 5; for rank 36: 3. Braton Prime rank 40 → `cap.rank` illegal
(MaxRank 30).

**TV-09-06 Rank scaling (E-9.13, E-9.12).**

| Input | Expected |
|---|---|
| Galvanized Chamber x_max 0.80, R = 10, r = 7 | 0.5818182 (PE shows 58.2 %); on-kill 0.30 → 0.2181818; with 5 stacks 1.6727273 |
| Condition Overload 0.80, R = 5, r = 2 | 0.40 |
| Bane of Grineer 0.30 bonus, R = 5, r = 2 | ×1.15 |
| Primed Bane of Grineer 0.55, R = 10, r = 4 | ×1.25 |
| Flawed Bane of Grineer 0.10, R = 3, r = 0 … 3 | ×1.025, 1.05, 1.075, 1.10 (PE shows 1.03, 1.05, 1.08, 1.1) |
| Internal Bleeding (affine) r = 0 … 5 | 0.10, 0.15, 0.20, 0.25, 0.30, 0.35 (not 0.0583 at r = 0) |
| Serration 1.65, R = 10, r = 3 | 0.60; drain 7 |
| Primary Merciless per stack 0.30, R = 5, r = 2 | 0.15; reload line absent (r < 5) |
| Sacrificial Steel Sentient bonus 0.33, R = 10, r = 4 | ×1.15 (PE ×1.15) |
| drains: Galvanized Chamber r 10; Blind Justice r 0; Steel Charge r 5; Synth Charge r 3 | 16; −2; −9; 9 |

**TV-09-07 Compatibility (E-9.4 to E-9.8).**

| Mod → weapon | Expected | Rule |
|---|---|---|
| Serration (Type Rifle) → Braton Prime (Primary/Rifle) | legal | |
| Primed Chamber (Type Sniper) → Braton Prime | illegal | `compat.type` |
| Primed Chamber → Rubico Prime (Sniper Rifle); Primed Chamber + Charged Chamber together | legal; legal | |
| Tainted Mag (Assault Rifle) → Paris (Bow) | illegal | `compat.type` |
| Split Flights (Bow) → Lenz (tags incl. CRPBOW) / → Paris (tags PROJECTILE) | illegal / legal | `compat.tags` |
| Sinister Reach (Primary, needs BEAM) → Amprex (BEAM) / → Braton Prime | legal / illegal | `compat.tags` |
| Synth Charge → Lex Prime (Magazine 8) / Furis (35) / Kulstar (3) / Knell (1, SINGLESHOT) / Epitaph (SINGLESHOT) / Balefire Charger (POWER_WEAPON) | legal / legal / legal (bug list) / illegal / illegal / illegal | `compat.magazine`, `compat.tags` |
| Semi-Rifle Cannonade → Latron ("Semi-Auto", Incarnon form "Semi-Auto") / Argonak ("Auto / Semi") / Nagantaka ("Semi / Mag Burst") | legal / illegal / illegal | `compat.semi` |
| Semi-Pistol Cannonade → Lex Prime / Laetum ("Semi / Auto") / Catchmoon (Secondary) | legal / illegal / legal | `compat.semi` |
| Semi-Shotgun Cannonade → Strun (Incarnon form inherits "Semi-Auto") / Catchmoon (Primary, MODULAR_GUN) | legal / illegal | `compat.semi`, `compat.tags` |
| Scattered Justice (Type Hek) → Hek / Kuva Hek / Vaykor Hek (SYNDICATEMODBLOCKED) | legal / legal / illegal | `compat.tags` |
| Blind Justice (stance, NIKANAS_STANCE) → Nikana Prime stance slot / a Nikana Prime general slot / Dragon Nikana stance slot | legal / illegal / legal | `slot.kind` |
| Bite (beast claw mod, Type Claws) → Garuda Talons (melee Claws) | illegal | `compat.type` |
| Sniper Ammo Mutation (needs SNIPER_AMMO or LAUNCHER_AMMO, exilus) → Rubico Prime exilus slot | legal | `compat.tags` (any of) |

**TV-09-08 Exclusion and duplicates (E-9.10, E-9.9).** On one weapon: Serration + Amalgam Serration → illegal
(`excl.pair`); Split Chamber + Galvanized Chamber → illegal; Critical Delay + Point Strike → illegal; Heavy Caliber +
Serration → legal; Vigilante Armaments + Split Chamber → legal; Electrified Barrel + Primed Electrified Barrel (only
the Primed mod lists the other) → illegal; Serration twice → illegal (`dup.mod`); two rivens → illegal
(`dup.riven`). On a warframe: Arcane Avenger twice → illegal (`dup.arcane`); five identical Tauforged Violet shards
with the primary-Electricity bonus → legal.

**TV-09-09 Set counting (E-9.40).** Warframe mods: Vigilante Pursuit, Vigilante Vigor. Primary: Vigilante Armaments,
Offense, Fervor, Supplies. Declared robotic-weapon pieces: Vigilante Armaments.

| Evaluated attack on | Expected N_Vigilante |
|---|---|
| the primary | 6 |
| the primary, robotic piece removed | 6 |
| the primary, warframe mods removed | 4 |
| the melee weapon (primary holstered) | 2 |
| the primary, Vigilante Armaments entered twice through a link (illegal duplicate) | 6 (identities counted once) |

Gladiator: Might, Vice, Rush on the melee weapon and Aegis, Finesse, Resolve on the warframe → 6 on a melee attack, 3
on a primary attack. Sacrificial Steel and Pressure on the melee weapon → 2.

**TV-09-10 Riven value ranges (E-9.53 to E-9.56; E-4.18 in binary32).**

| Riven | Trait | Layout, rank | Range of v (100·v %) | Input → result |
|---|---|---|---|---|
| Braton (Rifle) | Critical Chance b 0.016666 | 2+1, 8 | 200.4670 … 245.0152 | +220.0 → legal (u 0.4385) |
| Braton (Rifle) | Multishot b 0.01 | 2+1, 8 | 120.2850 … 147.0150 | +130.0 legal; +150.0 → illegal (u 1.1117) |
| Braton (Rifle) | Damage b 0.0183333 (negative) | 2+1, 8 | −107.8108 … −88.2088 | −95.0 → legal |
| Braton (Rifle) | Critical Chance | 2+1, 0 | 22.2741 … 27.2239 | — |
| Braton (Rifle) | Faction b 0.005 | 2+0, 8 | ×1.48114 … ×1.58806 | ×1.53 → legal |
| Braton (Rifle) | Faction (negative) | 2+1, 8 | ×0.70597 … ×0.75943 | — |
| Braton (Rifle) | Weak Point Damage splice b 0.025 | 2+1, 8 | 300.7125 … 367.5375 | — |
| Braton (Rifle) | Weapon Recoil b −0.01 (positive role) | 2+1, 8 | −147.015 … −120.285 | −130.0 → ρ = +1, legal |
| Catchmoon kitgun (Pistol column) | Damage b 0.0244 | 3+0, 8 | 177.876 … 217.404 | +195.0 → legal (u 0.4332) |
| Catchmoon kitgun | Damage | 3+0, 5 | 118.584 … 144.936 | — |
| a melee riven | Critical Chance b 0.02 | 2+0, 8 | 192.456 … 235.224 | — |
| a melee riven | Initial Combo b 0.27224 | 3+0, 8 | 19.8463 … 24.2566 (absolute) | — |

The Rifle rows reproduce the wiki overview table (`W:Riven_Mods` L56-80: Critical Chance 200.5 — 245.0, Damage
−88.2 — −107.8, Faction x1.48 — x1.59 and x0.71 — x0.76, Multishot 3+1 91.1 — 111.4).

**TV-09-11 Riven structure (E-9.50 to E-9.55).**

| Riven on weapon | Expected | Rule |
|---|---|---|
| Rifle riven, family Braton, on Braton Prime; CC+, MS+, Damage− | legal | |
| same riven on Mk1-Braton / on Braton Vandal | legal / legal | |
| Rifle riven family Cernos on Mutalist Cernos | illegal | `riven.binding` |
| Shotgun riven on Braton | illegal | `riven.binding` |
| Kitgun riven family Catchmoon on Catchmoon (Primary) and on Catchmoon (Secondary) | legal, Pistol column | |
| any riven on Exalted Blade | illegal | `riven.binding` (class none) |
| Rifle riven with 4 positive traits | illegal (P = 4) | `riven.layout` |
| Rifle riven with 2 negative traits | illegal (N = 2) | `riven.layout` |
| Rifle riven with Heat −40 % | illegal | `riven.trait` T3 |
| Melee riven with Multishot | illegal | `riven.trait` T2 |
| Rifle riven with Viral and Corrosive (two splices) | illegal | `riven.trait` T5 |
| Rifle riven with Parry Angle | illegal | `riven.trait` T5 |
| Rifle riven: Viral (splice) +, Cold +, Toxin + | legal | |
| Rifle riven with Impact + on Braton Prime (IMP 1.75 of 35 = 5 %) | legal with warning | `riven.physical` |

**TV-09-12 Valence (E-9.70, E-9.71).** Legal v10: 250, 579, 600; illegal: 249, 580, 595, 601. Fusion: (520, 400) →
572; (528, 250) → 600; (527, 527) → 579; (480, 300) → 528; (437, 437) → 480. Repeated self-fusion from a start value
until 600: 528 → 1 step, 480 → 2, 437 → 3, 398 → 4, 362 → 5, 330 → 6, 300 → 7, 273 → 8, 250 → 9; 436 → 4; 272 → 9.
Kuva Ghoulsaw (`Traits` contains Kuva Lich, `IsLichWeapon` absent) accepts a valence; Braton Prime with a valence →
illegal (`valence.weapon`); v10 = 580 → `valence.value`.

**TV-09-13 Archon shards (E-9.31).** (a) Shards: Violet (Tau, Electricity), Violet (Tau, Electricity), Crimson
(normal, Ability Strength), Azure (Tau, Health), Amber (normal, casting speed): legal; N_cav = 4. (b) Six shards →
illegal (`shard.count`). (c) Azure shard with "primary status chance" → illegal (`shard.bonus`). (d) Five Emerald shards with the
Corrosion max-stacks bonus → legal (repeats allowed).

**TV-09-14 Helminth (E-9.32).** (a) Self frame Rhino, subsumed Eclipse (self-cast), native Roar self-cast → illegal
(`helminth.host`). (b) Self frame Saryn, subsumed Roar and native Toxic Lash, both self-cast → legal (Saryn not a
host). (c) Self frame Chroma, subsumed Roar, ally-cast Vex Armor from another Chroma → legal. (d) Two subsumed
abilities → illegal (`helminth.count`). (e) Self frame Mesa, subsumed Xata's Whisper replacing Shatter Shield, native
Shooting Gallery → legal.

**TV-09-15 Incarnon (E-9.60).** Furis with `genesis_installed`: `inc2` = Haven Foray (X = 28), `inc3` = Extended
Volley, `inc4` = Elemental Balance (X = 0.28) → legal; T = 4, so `inc5` set → illegal (`incarnon.tier`). Mk1-Furis with the
same choices → legal with X = 34 and 0.39. Two perks for `inc2` → illegal (`incarnon.tier`). Any perk on Afuris
(Akimbo) → illegal (`incarnon.line`). Furis without `genesis_installed` → the "Incarnon Form" attack is not evaluable
and perks are illegal (`incarnon.form`).
Laetum (native): T = 5, `inc5` allowed.

**TV-09-16 Arcane slots (E-9.30).** Archgun (Imperator): `arcane0` Primary Merciless, `arcane1` Secondary Shiver →
legal; `arcane0` Secondary Shiver → illegal (`arcane.family`). Braton Prime `arcane1` set → illegal (`slot.count`). Catchmoon
(Secondary) kitgun: `arcane0` Secondary Merciless, `arcane1` Pax Charge → legal; `arcane1` Exodia Contagion →
illegal (`arcane.family`). Warframe: Arcane Avenger + Arcane Fury → legal; robotic weapon with any arcane → illegal
(`slot.count`). Primary Merciless
rank 6 → illegal (`rank.range`, R = 5); Pax Charge rank 4 → illegal (R = 3).
