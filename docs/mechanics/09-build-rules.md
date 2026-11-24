Status: reviewed (spec)

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
| `M:Mods`, `M:Weapons`, `M:Arcane`, `M:Modular` | wiki `Module:Mods/data`, `Module:Weapons/data/<slot>`, `Module:Arcane/data`, `Module:Modular/data`, as JSON in `research/external/cache/poc/out/` (`Mods_data.json`, `Weapons_data_*.json`, `Arcane_data.json`, `Modular_data.json`), read 2026-11-23 |
| `PE:Upgrades`, `PE:Arcane` | DE Public Export `research/external/cache/public-export/manifests/ExportUpgrades_en.json` and `ExportRelicArcane_en.json` (manifest of 2026-10-22, `DS:pe` §1.7) |
| `W:Semi-Pistol_Cannonade`, `W:Semi-Shotgun_Cannonade` | fetched for this document on 2026-11-23 and cached in `research/external/cache/wiki/` |
| `W:Haalvu`, `W:Dual_Heat_Swords` | fetched for the review on 2026-11-24 and cached in `research/external/cache/wiki/` |

Line numbers `L<n>` after a `W:` key refer to the cached raw wikitext file.

### 1.3 Inputs

| Input | Symbol | Data field (source) | Notes |
|---|---|---|---|
| Weapon record | W | `M:Weapons`: `Name`, `Slot`, `Class`, `Family`, `Traits`, `CompatibilityTags`, `Trigger` (weapon and per attack), `Magazine`, `MaxRank`, `Polarities`, `ExilusPolarity`, `StancePolarity`, `Attacks[]`, `InternalName`; module of origin (`primary`, `secondary`, `melee`, `archwing`, `companion`, `modular`) | `DS:wiki` §3; join key to PE is `InternalName` |
| Mod record | m | identity `uniqueName`, `polarity`, `baseDrain`, `fusionLimit`, `type`, `isUtility`, `modSet`, `levelStats` from `PE:Upgrades`; `Type`, `CompatibilityTags`, `IncompatibilityTags`, `Incompatible`, `Set` from `M:Mods` (joined on `InternalName`). Where both sources carry a field, PE wins (EC-09-73). A mod with no PE record uses the fallback of section 2.3 (E-9.3a); the hand-authored override tables X-STANCE (section 2.3) and X-EXALT (section 3.2) win over both | `DS:pe` §1.3, `DS:wiki` §4 |
| Kitgun base magazine | `Mag0` | `06` E-6.30 (`M:Modular` chamber `Magazine[loader tier]`) | E-9.7 (the chamber record's `Magazine = 0` is a placeholder) |
| Set table | `NumUpgradesInSet` | `M:Mods.Sets`; PE `ExportModSet` | `DS:wiki` §4, `DS:pe` §1.3 |
| Arcane record | a | `M:Arcane`: `Type`, `MaxRank`, `InternalName`; per-rank text `PE:Arcane` `levelStats` | `DS:wiki` §5 |
| Riven coefficients b and the value formula | `riven_coeff`, `riven_value` | `04` §8.2-8.3 (E-4.18); PE riven `upgradeEntries` | owned by `04` |
| Riven trait signs, class availability, splice recipes | – | hand-authored from `W:Riven_Mods` L809-1046, L1294-1398 | no data module (`DS:wiki` §4) |
| Incarnon perk table | – | hand-authored from `W:<Weapon>_Incarnon_Genesis` pages and native weapon pages | absent from every data source (`DS:wiki` §3, `DS:wfcd` §2.5, `DS:pe` §1.6) |
| Archon shard bonus table | – | hand-authored from `W:Archon_Shard` L244-400 | absent from every data source (`DS:pe` §1.6) |
| Helminth restriction table | – | hand-authored from `W:Helminth` L259-278 | |
| Listed base damage per type of the weapon's default attack | `base_listed[t]` (B0_t) | `M:Weapons` `Attacks[]` (`01`) | physical share of E-9.57 |
| Caster of each buff (self or ally) | `buff_str` caster | `04` §4.4 (catalogue field `caster`); for the toggle T-13 (Roar) the caster is T-37 (`self` / `ally`, default `ally`); T-14 … T-16 (Eclipse, Xata's Whisper, Toxic Lash) are always self-cast (section 10) | Helminth rule E-9.32 |
| Companion kind | `companion_kind` | user input: `sentinel`, `kubrow`, `kavat`, `predasite`, `vulpaphyla`, `moa`, `hound`, `none` (default `none`) | E-9.41 |
| Toggles | – | T-02; T-13 with its caster T-37; T-14 … T-16 (self-cast); T-24; T-26; T-35 capacity mode (with `frame_complete`); T-36 lower mod ranks in the optimizer | overview section 6.1 (adopted from OQ-915) |

### 1.4 Outputs and symbols added

Outputs: the legal content of every slot, each mod's rank and the effect value its rank implies (E-9.13), the set counts
`set_count[S]` read by `04` §9, the Archon shard list read by `04` §5.9, the valence fraction read by `01`, the
selected Incarnon perks read by `04` §5.12, the capacity and Forma report, and the list of violated rules.

Symbols added (domain prefixes per overview section 0.3). Math symbols avoid the letters the overview's symbol table
already assigns (F faction, C CO bucket, ρ duty, s status type, d distance, k crit tier); the rank symbol r is the
one `04` already uses for mod, arcane and riven ranks (`04` §1.5, E-4.18) and always appears without a type subscript,
so it does not clash with the overview's ratio r_t.

| ID | Symbol | Unit | Meaning |
|---|---|---|---|
| `bld_slot` | ω | enum | a slot of an item (section 2.1) |
| `mod_rank`, `mod_rank_max` | r, R | integer | rank of a mod or arcane instance; maximum rank (`fusionLimit`, `M:Arcane.MaxRank`) |
| `mod_drain` | ξ(r) | capacity points | drain at rank r before polarity (negative for aura and stance mods) |
| `riv_drain0` | ξ0 | capacity points | a riven's drain at rank 0 (section 12.4) |
| `pol_slot`, `pol_mod` | π_ω, π_m | Polarity | polarity of a slot, of a mod |
| `cap_slot_cost` | c_ω | points | capacity charged by slot ω after polarity (E-9.14) |
| `cap_bonus` | β | points | capacity added by the stance (weapon) or aura (warframe) slots (E-9.15) |
| `cap_item` | C_cap | points | item capacity (E-9.16) |
| `cap_used` | U | points | `Σ c_ω − β` (E-9.17) |
| `forma_count` | N_forma | count | Forma needed to reach the evaluated polarity layout (E-9.19), reported per Forma type; `N_forma*` the minimum |
| `cap_mode` | – | enum | capacity constraint mode `ignore`, `current`, `any_forma`, `budget(N)` (E-9.18; toggle T-35) |
| `exalt_req(m)` | – | set of tags | weapon tags that make mod m unequippable on an exalted weapon (X-EXALT, E-9.5) |
| `set_count[S]` | N_S | count | pieces of set S that count (E-9.40) |
| `companion_kind` | – | enum | kind of the companion that carries declared set pieces (E-9.41) |
| `riv_class` | – | enum | `Rifle`, `Shotgun`, `Pistol`, `Kitgun`, `Melee`, `Zaw`, `Archgun`, `Companion`, `none` |
| `riv_family` | – | string | normalised weapon family a riven binds to (E-9.51) |
| `riv_role[i]` | sgn_i | ±1 | +1 for a bonus (positive) trait, −1 for a curse (negative) trait |
| `riv_layout` | (n_pos, n_neg) | counts | number of positive and negative traits |
| `val_tenths` | v10 | integer | progenitor value in units of 0.1 % (`progenitor_frac = v10 / 1000`) |
| `inc_choice[t]` | – | perk id or none | selected perk of Incarnon evolution t (default none) |
| `shard[z]` | (colour, tau, bonus) | – | Archon shard z (z = 0 … 4) of the warframe |
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
   constraint groups. The groups, by the key prefix of their rules:

   | Group | Keys | Can be disabled |
   |---|---|---|
   | slots | `slot.count`, `slot.kind` | no |
   | compatibility | `compat`, `compat.type`, `compat.tags`, `compat.semi`, `compat.magazine`, `arcane.family`, `arcane.gate` | no |
   | exclusion and duplicates | `excl.pair`, `dup.mod`, `dup.arcane`, `dup.riven` | no |
   | ranks | `rank.range` | no (lower ranks are offered to the optimizer only under T-36) |
   | capacity | `cap.rank`, `cap.exceeded` | yes: mode `ignore` of `cap_mode` (E-9.18, toggle T-35) |
   | shards, Helminth, sets | `shard.*`, `helminth.*`, `set.companion` | no |
   | rivens | `riven.binding`, `riven.layout`, `riven.trait`, `riven.range` | no; it constrains rivens the optimizer generates; a user riven is never rewritten (rule 3) |
   | Incarnon, valence | `incarnon.*`, `valence.*` | no |

   Rules of severity `warning` (`riven.physical`, `valence.unset`) are never constraints.
3. Items the user supplies (a riven, a valence value, locked slots) are never rewritten by the optimizer. If they
   violate a rule, the optimizer still runs and its report repeats the violation.
4. A rule whose input field is missing in the data is evaluated as stated for the missing value and the data-build
   report lists the weapon or mod (`docs/data/`). Fields with a stated fallback (E-9.3a mod records without PE, the
   `Family` fallback of E-9.51, the slot polarity strings of section 6.1) use that fallback instead.

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
the inspiration listing the Incarnon-form attack, `I:feat` §4.1), `inc_choice[2..5]` (default none: an evolution
without a stated perk contributes nothing), `valence` (element, v10), kitgun and zaw parts (`06`, `07`). Warframe
item: `frame` (optional identity), `reactor` (default true), polarities, `frame_complete` (default false, section 6.5).
Scenario: `companion_kind` (default `none`) for the declared companion set pieces of section 11.

### 2.2 Equipment class and slot counts

```text
E-9.1  Equipment class of a weapon record W                                  key: data only
  exalted(W) = 'POWER_WEAPON' ∈ W.CompatibilityTags ∨ W.Class = 'Exalted Weapon'
  pseudo(W)  = 'POWER_WEAPON_LITE' ∈ W.CompatibilityTags                    (pseudo-exalted melee)
  kitgun(W)  = W comes from Module:Weapons/data/modular ∧ W.Slot ∈ {Primary, Secondary}
  zaw(W)     = W is an assembled zaw (07); its record comes from Module:Weapons/data/modular with Slot Melee
  kitgun fields: Class, Trigger, CompatibilityTags (plus derived(W) of E-9.5) and Family come from the chamber
           record of Module:Weapons/data/modular for the chosen slot ("Catchmoon (Secondary)"); the base magazine
           is 06's assembled Mag0 = chamber.Magazine[loader tier] (06 E-6.30), never the chamber record's
           placeholder Magazine = 0; the pre-expanded Module:Modular/data/kitgun combos are not used (their
           CompatibilityTags are empty, 06 §4.1)
  cls(W) = archgun    if W.Slot ∈ {Archgun, Archgun (Atmosphere)}
           archmelee  if W.Slot = Archmelee
           robotic    if W.Slot ∈ {Robotic, Hound}
           beast      if W.Slot = Beast                                     (out of scope, overview 1.3)
           primary | secondary | melee   if W.Slot = Primary | Secondary | Melee
```

Sources: `DS:wiki` §3 (fields, module list); slot values counted in `M:Weapons` (archwing: `Archgun` 20, `Archgun
(Atmosphere)` 21, `Archmelee` 9; companion: `Robotic` 21, `Beast` 17, `Hound` 3); the four pseudo-exalted weapons
(Landslide Fists, Shadow Clones, Shattered Lash, Whipclaw and their Primes) are exactly the records with
`POWER_WEAPON_LITE` (`W:Exalted_Weapon` L74-88); since U38.5 they are exalted weapons too and also carry
`POWER_WEAPON` (`W:Exalted_Weapon` L227, L312). The `Class` test catches the Necramech exalted weapons Arquebex and
Ironbride, whose records have `Class = 'Exalted Weapon'` but no tags (`W:Exalted_Weapon` L68-70; EC-09-80). Kitguns are
recognised by origin, not by tag: only 6 of the 12 chamber records carry `MODULAR_GUN` (EC-09-15); all 12 chamber
records have `Magazine = 0` (EC-09-69).

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
| exalted primary or secondary | 8 | 1 | – | 1 (as its slot) | Catalyst pre-installed (`W:Exalted_Weapon` L10); exilus and arcane slot since U38.5 (L233-249) |
| exalted melee | 8 | 1 | 1, fixed Zenurik stance | 1 (melee) | arcane slot since U38.5 (L239) |
| pseudo-exalted melee | 8 | 0 | 1, fixed | 1 (melee) | no exilus (`W:Exalted_Weapon` L96, L314); arcane slot: "the same treatment" (L312) |
| Garuda Talons (class Claws, not exalted) | 8 | 1 | 1, free (any claw stance, Madurai) | 1 (melee) | an ordinary melee weapon for E-9.2 (`W:Exalted_Weapon` L95); the arcane slot was added explicitly in U38.5 (L241); no rivens (E-9.50) |
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
  kind(m) = stance  if m ∈ X-STANCE (below)
            kind of E-9.3a if m has no PE record
            aura    if PE type = AURA
            stance  if PE type = STANCE
            riven   if m is a riven instance (section 12)
            general otherwise;   exilus_ok(m) = PE isUtility = true (E-9.3a without a PE record)
  gen0..gen7, fgen0..fgen7 accept kind ∈ {general, riven}  (exilus-capable mods included)
  exilus, fexilus          accept kind = general ∧ exilus_ok(m)
  stance                   accepts kind = stance; if stance_fixed(W) only the weapon's own exalted stance
  aura0, aura1             accept kind = aura
  rivens are never exilus-capable (they have no isUtility flag)

E-9.3a Mod records without a PE record (joined on M:Mods.InternalName)       key: data only
  identity  = M:Mods.InternalName (a record without InternalName has no identity and is never offered)
  kind      = stance if 'MeleeTree' ⊂ InternalName; aura if M:Mods.Type = 'Aura'; general otherwise
  R         = M:Mods.MaxRank;  baseDrain = M:Mods.BaseDrain;  polarity = M:Mods.Polarity (section 6.1)
  exilus_ok = M:Mods.IsExilus = true
  slot_type = the slot of the compatibility tree (section 3.1) that holds norm_type(M:Mods.Type), as in E-9.4
  the data-build report lists every such mod

X-STANCE  Exalted stances (hand-authored; W:Exalted_Weapon L94, L205-210, L280)
  members: the stance mods whose M:Mods.Type is an exalted family: Exalted Blade, Primal Fury (Iron Staff), Serene
           Storm (Desert Wind), Razorwing (Diwata), Hysteria (Valkyr Talons), Ravenous Wraith (Shadow Claws), Blade
           Storm (Shadow Clones), Landslide (Landslide Fists), Shattered Lash, Whipclaw
  kind = stance; polarity Zenurik; baseDrain −2; rank fixed at R = 3, so ξ(3) = −5 and b = 5 (E-9.15);
  equippable only in the stance slot of the weapon whose Family is its Type (E-9.8)
```

Sources: `W:Mod/Compatibility` L286-289 (aura and stance slots exclusive; exilus mods anywhere, only exilus mods in the
exilus slot); `PE:Upgrades` `type` values AURA 36, STANCE 84 and `isUtility` 195 (`DS:pe` §1.3). The exilus flag comes
from PE `isUtility`, not from wiki `IsExilus`, which disagrees on 7 mods (EC-09-10).

The fallback E-9.3a is needed because `M:Mods` has mods that `PE:Upgrades` lacks (measured for this review by joining
on `InternalName`: 102 entries, of which 53 are outside the Railjack and Plexus types). Weapon-relevant ones: the ten
exalted stances, the stances Butcher's Revelry and Clashing Forest, Primed Deadly Efficiency, Primed Steady Hands,
Magnetic Rush, Magnetic Strafe and nine Flawed mods (EC-09-67). The `MeleeTree` test finds all 68 stance records of
`M:Mods` that carry the substring (56 of them have a PE `STANCE` record; the PE PvP stances lack the substring but have
PE records). The exalted stances need X-STANCE on top: their `M:Mods` records say `BaseDrain 0`, `MaxRank 3`,
`NotUpgradable`, which under E-9.12 would make them drain +3 instead of granting capacity, while U38.5 states "All
Exalted Stances now grant Mod Capacity just like regular Stances … Zenurik Polarity … providing a +10 capacity
boost" (`W:Exalted_Weapon` L280; patch note, precedence 2 over the data module, overview section 1.4). Regular stances
have `baseDrain −2` and max rank 3 in PE (`W:Stance` L710: bonus 5 when maxed), which X-STANCE copies.

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
| 18 | Incarnon evolution I (form; no choice) |
| 19-22 | `inc2` … `inc5` (`{mId, pId}` = tier and perk index); an evolution slot left empty imports as `inc_choice = none` |
| 23-27 | `shard0` … `shard4` (each shard entry maps to colour, tau and bonus; `I:model` §4.4) |
| 28, 29 | archgun `arcane0`, `arcane1` |
| `"Riven"` in 0-7 | the riven in that general slot, traits from link key `r` (`JS L9751-9765`): each row `r[k] = {t, p, a, v}` gives the trait key `p` and the displayed value `v`, converted by `04` §8.2 |

`genesis_installed` is true after import whenever any of the slots 18-22 is filled (a perk implies the Genesis; a link
with perks in 19-22 but no slot 18 is read as installed), and otherwise keeps its default of section 2.1 (true for a
weapon with an Incarnon line, because the inspiration lists the Incarnon-form attack regardless of slot 18); the
import never sets it to false (Decision). The trait keys `p` are the keys of the inspiration's riven catalogue (`JS
L14696-14738`) and map one to one: `crit_chance`, `crit_mult`, `multishot`, `status_chance`, `base` (Damage, or Melee
Damage on melee), `speed` (Fire Rate / Attack Speed), `Heat`, `Cold`, `Electricity`, `Toxin`, `Impact`, `Puncture`,
`Slash`, the six splices `addBlast`, `addCorrosive`, `addGas`, `addMagnetic`, `addRadiation`, `addViral`,
`status_damage` (splice), the factions `Corpus`, `Grineer`, `Infested` and the splices `Scaldra`, `Techrot`,
`crit_chance_weakp` (Weak Point Critical Chance splice; D-010), `mult_for_head` (Weak Point Damage splice, which the
inspiration files under the headshot bucket; D-011), `ammoEff` (Ammo Efficiency splice), `ammoCapacity`,
`magazineSize`, `reloadTime`, `shot_speed`, `comboDuration`, `initialCombo`, `melee_combo_eff` (Heavy Attack
Efficiency), `range`, `punch_through`, `zoom`, `recoil`, `status_duration`. The import keeps the trait even where our
catalogue says the riven's class cannot roll it (for example `zoom` on a melee riven, which the inspiration offers)
and flags it under `riven.trait` T2.

---

## 3. Mod compatibility with an item

A mod may go on a weapon only if its slot type (PE `type`, with the fallback of E-9.4) matches the item, its wiki
`Type` names a node of the weapon's compatibility tree, its tags allow the weapon (with the exalted-weapon overrides
X-EXALT), and any special requirement holds.

### 3.1 Compatibility tree

`W:Mod/Compatibility` L4-201 describes the tree slot → class → sub-class → family → specific weapon; a mod's
`Type` names the node it attaches to and the mod is allowed on every weapon below that node. Kitguns inherit the class
of their chamber, zaws the melee class of their strike (L117, L146, L174).

```text
E-9.4  Compatibility tree                                                    key: compat.type   severity: illegal
  pe_type_ok(m, W):  slot_type(m) = PRIMARY for cls primary; SECONDARY for secondary; MELEE for melee general and
                     exilus slots (STANCE for the stance slot, E-9.3); ARCH-GUN for archgun; ARCH-MELEE for archmelee;
                     robotic weapons: the type of their class row below
  slot_type(m) = PE type(m)                          if it is present and not '---'
               = the slot whose tree (table below) holds norm_type(M:Mods.Type), else of PE compatName
                                                     if PE type is '---' or missing (E-9.3a)
                 (a Type naming one weapon or family maps to that weapon's M:Weapons.Slot)
  nodes(W) = chain(W) ∪ {W.Family, W.Name}
  type_ok(m, W) = pe_type_ok(m, W) ∧ norm_type(m.Type) ∈ nodes(W)
```

PE has 80 records with `type = '---'`; the only weapon mods among them are Sniper Ammo Mutation, its Intermediate
copy and Primed Sniper Ammo Mutation (`compatName` "Sniper", `isUtility` true; `M:Mods.Type` "Sniper"), which
`slot_type` maps to PRIMARY. The other `---` records are Necramech (28), K-Drive (23), Antiques and the companion riven
stub (`compatName` ANY, 22) and transmute cores (4); a further 17 records (the riven stubs and Railjack artifacts) have
no `type` at all (counted for this review; EC-09-66). TV-09-07 holds the regression case.

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
  inc_tags(m, W) = m.IncompatibilityTags \ {'POWER_WEAPON'}   if m ∈ X-EXALT.lift(W)
                 = m.IncompatibilityTags                       otherwise
  tags_ok(m, W) = (m.CompatibilityTags = ∅ ∨ m.CompatibilityTags ∩ tags(W) ≠ ∅)
                ∧ (inc_tags(m, W) ∩ tags(W) = ∅)
                ∧ (¬exalted(W) ∨ exalt_req(m) ∩ tags(W) = ∅)
```

**X-EXALT, exalted-weapon overrides** (hand-authored; the wiki data module has not caught up with U38.5 and U38.6,
EC-09-68). `lift(W)` lists the mods whose `POWER_WEAPON` incompatibility tag is removed; `exalt_req(m)` lists the
weapon tags that still make m unequippable on an exalted weapon because the weapon "is unable to trigger the
requirements for the mod" (`W:Exalted_Weapon` L276).

| Mods | lift(W) | exalt_req(m) | Source |
|---|---|---|---|
| Amalgam Serration, Argon Scope, Galvanized Scope, Bladed Rounds, Catalyzer Link (primary) | every exalted W | Argon Scope, Galvanized Scope, Bladed Rounds, Catalyzer Link: `NO_AIM` ("when Aiming") | `W:Exalted_Weapon` L253-259 (U38.5) |
| Amalgam Barrel Diffusion, Hydraulic Crosshairs, Sharpened Bullets (secondary; Embedded Catalyzer, Pressurized Magazine, Targeting Subsystem carry no `POWER_WEAPON` tag) | every exalted W | Hydraulic Crosshairs, Sharpened Bullets, Embedded Catalyzer, Pressurized Magazine, Targeting Subsystem: `NO_AIM` | L260-266, L276 ("Example: Hydraulic Crosshairs on Mesa's Regulators") |
| Amalgam Organ Shatter, Blood Rush, Body Count, Gladiator Rush, Maiming Strike, Weeping Wounds (melee; Dispatch Overdrive carries no `POWER_WEAPON` tag) | every exalted W | Maiming Strike: `NO_SLIDE` ("for Slide Attack") | L267-274, L276 |
| Energizing Shot | every exalted W | – (its `AOE` incompatibility tag still excludes Glory, Balefire Charger, Noctua) | L156 (U38.6: "excluding … Glory, … Balefire and … Noctua since they are AOE weapons") |
| Harkonar Scope | Neutralizer only | – | L299 (U38.5: "Enabled Harkonar Scope on Cyte-09's Neutralizer") |

Requirement tags on weapons (`M:Weapons`, counted for this review): `NO_AIM` on Glory, Regulators, Regulators Prime;
`NO_SLIDE` on the eight pseudo-exalted records; `NO_FINISHER` and `NO_ATTACK_SPEED` on Landslide Fists, Shattered
Lash, Whipclaw and their Primes. No mod in `M:Mods` references these tags, so `exalt_req` is the only place they act.
Mods that need a heavy attack already carry `NO_HEAVY_ATTACK` or `POWER_WEAPON_LITE` in `M:Mods`. `NO_FINISHER` and
`NO_ATTACK_SPEED` have no mod in the table yet (no mod of the U38.5 list depends on finishers or attack speed); the
arcane rule E-9.30 uses `NO_FINISHER`. Galvanized Crosshairs, the Galvanized counterpart of Hydraulic Crosshairs, is
not named in U38.5 (unlike Galvanized Scope next to Argon Scope) and keeps its `POWER_WEAPON` tag (Decision,
UNVERIFIED, OQ-919). Data-build check: every mod of the table must still carry `POWER_WEAPON` in `M:Mods` (else the
override is stale and the report says so), and every other mod whose `POWER_WEAPON` tag conflicts with a newer patch
note found during maintenance is added here, never patched in the data.

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
  semi_only(W) = ∀ attack a of W that the build can use (normal, alt-fire, child attacks, and the Incarnon-form
                 attacks only while evolution I is available, E-9.60):
                   ∀ c ∈ components(a): c ∈ {'Semi-Auto', 'Semi'}
  The mods require 'SEMI_AUTO' (their CompatibilityTags); E-9.5 derives it from semi_only(W).
  Semi-Rifle and Semi-Shotgun Cannonade carry IncompatibilityTags MODULAR_GUN (primary kitguns excluded);
  Semi-Pistol Cannonade does not (secondary kitguns allowed since U37).
  'Duplex' is not Semi-Auto (Decision, UNVERIFIED, OQ-907).
  A Genesis weapon without genesis_installed is judged without its Incarnon-form attacks (Decision, UNVERIFIED,
  OQ-923): the pages tie the rule to weapons "with an Incarnon mode" (W:Semi-Rifle_Cannonade L29).

E-9.7  Synth Charge                                                         key: compat.magazine severity: illegal
  synth_ok(W) = Mag0(W) ≥ 6 ∨ W.Name ∈ {Kulstar}       (base magazine, never modded)
  Mag0(W) = W.Magazine of the weapon record; for kitgun(W) the assembled base magazine of the chosen chamber and
            loader (06 E-6.30), so legality depends on the loader (Decision, UNVERIFIED, OQ-920)
  plus E-9.5 with its IncompatibilityTags SINGLESHOT, POWER_WEAPON, SENTINEL_MOD
```

Sources: `W:Cannonade_Mods` L1-5 ("all of the listed trigger types must be Semi"; Argonak excluded, Latron allowed);
`W:Semi-Rifle_Cannonade` L28-31 (Incarnon modes, Nagantaka excluded); `W:Semi-Pistol_Cannonade` L30 (Bronco, Lato,
Lex Incarnon allowed), L35-40 (EFV-8 Mars excluded U38.5.10; secondary kitguns fixed U37);
`W:Semi-Shotgun_Cannonade` L29-30 ("Strun Incarnon Genesis is currently the only Incarnon weapon capable of equipping
this mod"); `M:Mods` (tags quoted). The per-attack `Trigger` field is missing on most Incarnon-form attacks (Strun,
Bronco, Lato, Vectis); inheriting the weapon trigger reproduces every case the three pages name (Latron, Vectis,
Strun, Bronco, Lato, Lex allowed; Argonak, Nagantaka, EFV-8 Mars excluded), checked against `M:Weapons` for this
document. The pages name no further exclusions (`W:Semi-Pistol_Cannonade` L30 lists examples with "such as"); the
exclusions of Laetum ("Semi / Auto"), Vasto and Vasto Prime (Incarnon Form "Burst"), Torid (Incarnon Form "Held") and
Dual Toxocyst (Incarnon Form "Auto") are derived from their `M:Weapons` triggers by E-9.6, not listed on the pages.
Synth Charge: `M:Mods.Description` and every PE rank text "Requires Magazine 6 or higher"; `W:Synth_Charge` L27 ("based
on the base Magazine Capacity of a gun"; raising the magazine does not qualify a weapon; lowering it later does not
disqualify), L28 (allowed on the Burst Laser), L29 (not on exalted weapons), L41 (Kulstar bug); U24.0.1 note (then "5
or higher") superseded. No source mentions kitguns; the six kitgun chambers' `M:Weapons` records all hold the
placeholder `Magazine = 0`, which would forbid Synth Charge on every secondary kitgun, so E-9.7 reads the assembled
magazine instead (EC-09-69): Catchmoon with the Zip loader (tier Lowest, 5 rounds) fails, with Arcroid (Med, 9) it
passes (`M:Modular` KitgunSecondary; TV-09-07).

```text
E-9.8  Mod compatibility                                                    key: compat           severity: illegal
  compatible(m, W) = type_ok(m, W) ∧ tags_ok(m, W) ∧ (m ≠ Synth Charge ∨ synth_ok(W))
                     (the Cannonade rule E-9.6 acts through the derived SEMI_AUTO tag inside tags_ok)
  stance mods: cls(W) = melee ∧ kind(m) = stance ∧ m.CompatibilityTags ∩ tags(W) ≠ ∅  (type_ok not required;
               a stance without tags, such as a beast posture, never fits; Garuda Talons accept any claw stance,
               W:Exalted_Weapon L95)
               if stance_fixed(W): the only legal stance is the X-STANCE mod whose M:Mods.Type = W.Family (Exalted
               Blade, Primal Fury, Serene Storm, Razorwing, Hysteria, Ravenous Wraith, Blade Storm, Landslide,
               Shattered Lash, Whipclaw); X-STANCE mods carry no CompatibilityTags and are matched by this rule only
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
graph has 3 non-transitive triples, all of Conclave rifle mods: Double Tap ⟂ Hydraulic Gauge, Loose Hatch or Maximum
Capacity ⟂ Precision Munition, while Double Tap and Precision Munition are not listed against each other. Double Tap
is Latron-only and Precision Munition Tonkor-only (`M:Mods.Type`), so the end mods can never share a weapon and a
closure would change no buildable weapon. Families covered by the lists: normal, Flawed, Primed, Umbra, Amalgam,
Galvanized and Archon variants of one mod, plus the oddities of `F:mods` §5.1 (Critical Delay ⟂ Point Strike;
Critical Deceleration ⟂ Blunderbuss; Creeping Bullseye ⟂ Pistol Gambit; Berserker Fury ⟂ Fury; Split Flights ⟂ Split
Chamber, Galvanized Chamber; Argon Scope ⟂ Galvanized Scope; Sacrificial Steel ⟂ True Steel; Sacrificial Pressure ⟂
Pressure Point; Spectral and Higasa Serration ⟂ Serration) and pairs found only in the data (Concealed Explosives ⟂
Energizing Shot, `M:Mods.Incompatible` of both). Pairs that **do** stack are simply absent from the lists (Charged
Chamber + Primed Chamber, Heavy Caliber + Serration, Spoiled Strike + Pressure Point, Vigilante Armaments + Split
Chamber, Augur Pact + Hornet Strike; `F:mods` §5.1, `W:Mod` L272-294 "Nonstandard Mods").

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
  mods:    r ∈ [0, R], R = PE fusionLimit (3, 5 or 10 for weapon mods; stances 3; auras 5); without a PE record
           M:Mods.MaxRank (E-9.3a); X-STANCE stances are fixed at r = 3
  arcanes: r ∈ [0, R], R = M:Arcane.MaxRank (5; 3 for Kitgun, Zaw and Amp Virtuos arcanes)
  rivens:  r ∈ [0, 8]
  default r = R; the optimizer uses R unless T-36 allows lower ranks
```

Sources: `F:mods` §3, §11; `W:Arcane_Enhancement` L28-53; `M:Arcane` (`Type`/`MaxRank`: Kitgun 3, Zaw 3, Amp 3 or 5,
all others 5); `W:Riven_Mods` L810 ("for rank 8 (max rank) Rivens").

### 5.2 Drain

```text
E-9.12 Drain at rank r (before polarity)
  ξ(r) = baseDrain + r      if baseDrain ≥ 0
  ξ(r) = baseDrain − r      if baseDrain < 0     (aura and stance mods: a negative drain adds capacity)
  baseDrain from PE:Upgrades (E-9.3a without a PE record; X-STANCE for exalted stances; rivens: section 12.4)
```

Sources: `F:mods` §2.2 (Serration 4 → 14, Galvanized Chamber 6 → 16, Condition Overload 10 → 15); `W:Synth_Charge`
rank table (cost 6, 7, 8, 9); stances `baseDrain` −2 with max rank 3 give −5 (`F:mods` §2.1, `W:Stance` L710 "bonus
mod capacity of 5 when maxed"); Steel Charge −4 at rank 5 gives −9 (`W:Orokin_Reactor` L54, "+9 Max Rank Steel
Charge"). Riven drain is not in any source; section 12.4 defines its input and default (OQ-905).

`baseDrain` and the maximum rank come from PE, not from `M:Mods`: the two disagree on 44 non-riven records (38 of them
primary, secondary or melee mods), and on the checkable cases PE is the game: `W:Mod` L423 says Cryo Rounds has
drained 11 at maximum rank since U16, which PE gives (6 + 5) and `M:Mods` does not (2 + 5). Other weapon examples:
Point Strike and Vital Sense 4 (PE) vs 2 (`M:Mods`), Split Chamber 10 vs 4, Infected Clip 6 vs 4, Reach maximum rank 3
vs 5 (EC-09-73).

### 5.3 Per-rank effect values

```text
E-9.13 Per-rank value of a numeric field of a stat line (mods and arcanes), maximum-rank value x_max
  scope: every numeric field of a PE levelStats line (PE:Upgrades, PE:Arcane) whose value differs between ranks
         (amounts, caps, thresholds, durations, stack caps, radii); a field that is the same at every PE rank is a
         constant and takes x_max at every rank
  x(r) = x_max · (r + 1) / (R + 1)                         default (computed in f64 from the maximum value)
  multipliers displayed "×m": the rule applies to b = m − 1 (Bane ×1.05 … ×1.30; Flawed Bane ×1.025 … ×1.10;
         Sacrificial Sentient ×1.03 … ×1.33; Melee Careen ×1.25 … ×2.50)
  exceptions (table X-RANK below): affine x(r) = x_0 + (x_max − x_0)·r / R, or an explicit per-rank table
  rank-gated lines: present only from a given rank (X-RANK)
  set bonuses do not depend on rank (section 11)
```

**X-RANK, per-rank exceptions** (this document owns the list, overview section 7.1 "mod rank scaling"; `04` §1.5
points to this table as the single owner and its catalogue rows cite it, OQ-925). Damage-relevant fields only; series
read from `PE:Upgrades` and `PE:Arcane` for this review.

| Item | Field | Rule (r = 0 … R) | PE series |
|---|---|---|---|
| Internal Bleeding, Hemorrhage, Magnetic Welt | proc chance | `0.10 + 0.05·r` | 10 … 35 % |
| Critical Precision (Tiberon) | crit chance per headshot | `0.025 + 0.015·r` | 2.5, 4, 5.5, 7, 8.5, 10 % |
| Clip Delegation (Sobek) | status chance and multishot per shot landed | `0.05 + 0.02·r` | 5 … 15 % |
| Necrophagic Vigor (Hema), Dreadful Killshot (Basmu) | bonus per 20 health drained / per 75 health | `0.10 + 0.02·r` | 10 … 20 % |
| Volatile Variant (Sporothrix) | status chance of direct barbs | `0.25 + 0.05·r` | 25 … 50 % |
| Unseen Dread (Dread) | crit damage while invisible | `0.50 + 0.25·r` | 50 … 175 % |
| Bursting Mass (Mutalist Quanta) | share of damage stored | `0.50 + 0.30·r` | 50 … 200 % |
| Static Discharge (Prova) | share of weapon damage on nearby enemies | `0.30 + 0.10·r` | 30 … 80 % |
| Exposing Harpoon (Harpak) | crit chance after a pull | table 1.00, 1.30, 1.60, 2.00, 2.50, 3.00 | |
| Wild Frenzy (Grakata) | alt-fire fire rate; ammo refund | table 2.00, 2.22, 2.50, 2.85, 3.33, 4.00; refund `0.25 + 0.15·r` | |
| Truth's Flame | Tennokai damage | table 0.30, 0.40, 0.60, 1.20 | |
| Double Tap (Latron) | per-stack bonus; duration; stack cap | `0.05·(r + 1)`; `0.5·(r + 1)` s; table 80, 40, 26, 20 | |
| Gorgon Frenzy (Gorgon) | fire rate; duration | proportional; table 0.75, 2, 2, 3 s | |
| Spring-Loaded Broadhead (Daikyu) | distance threshold | `45 − 6·r` m | 45 … 15 m |
| Melee Duplicate | chance per base crit | `0.25 + 0.15·r` | 25 … 100 % |
| Melee Doughty | crit multiplier per 10 % Puncture status | `0.5 + 0.1·r` | |
| Secondary Fortifier | overguard multiplier | `×(3 + r)` (b = 2 + r) | ×3 … ×8 |
| Secondary Surge | cap | `×(3 + r)` (b = 2 + r, `04` cap_r) | ×3 … ×8 |
| Primary Compression | damage per metre of radius lost; ammo efficiency per metre | `0.50 + 0.10·r`; `0.030 + 0.005·r` | |
| Primary Bulwark | cap (rate 0.01 per armor point constant) | `2.50 + 0.50·r` | 250 … 500 % |
| Primary Overcharge | multishot cap (rate 0.35 of max energy constant) | `1.00 + 0.50·r` | 100 … 350 % |
| Cascadia Empowered | flat damage per status effect | `250 + 100·r` | 250 … 750 |
| Primary Debilitate | chance | `0.50 + 0.10·r` | 50 … 100 % |
| Secondary Outburst | crit chance and crit damage per combo consumed | table 0.05, 0.07, 0.09, 0.12, 0.16, 0.20 | |
| Secondary Irradiate | share of hit damage on nearby enemies; radius | `0.80 + 0.20·r`; `4.5 + 0.5·r` m | (multi-target) |
| Secondary Enervate | big crits before reset | `r + 1` | (`04` E-4.21) |
| Primary Plated Round | duration | table 3, 4, 5, 6, 8, 10 s | |
| Primary / Secondary Merciless | reload line +0.30 | rank-gated: r = 5 only | |
| Primary / Secondary Deadhead | headshot-multiplier and recoil lines | rank-gated: r = 5 only | |
| Primary / Secondary Dexterity | combo-duration line | rank-gated: r = 5 only | |
| warframe arcanes | "+1 Arcane Revive" | rank-gated: r ≥ 3 (for example Arcane Crepuscular, Arcane Avenger) | |

Sources: `F:mods` §3 (Serration, Primed Pressure Point, Condition Overload, Critical Delay, Galvanized Chamber, Bane;
Merciless), `DS:wfcd` §3.4 (derive lower ranks from the maximum, displayed values are rounded); the X-RANK series and
gates are the PE `levelStats` texts of each item. Proportional series that only look irregular because DE rounds or
truncates the display (Primary Acuity shows 254 where 350·8/11 = 254.5; Secondary Kinship 3, 6, 10, 13, 17, 20 for
`0.20·(r + 1)/6`) follow the default. Arcane Crepuscular's final crit damage shows +1, +1, +2, +2, +3, +3 ×, which
is `0.5·(r + 1)` rounded half up and is therefore treated as proportional (UNVERIFIED; a step function would show the
same text). The engine does not round internal values to the displayed decimals (Decision; `F:mods` §3 "compute from
max-rank value"). Regression test: every numeric field of every `levelStats` line of `PE:Upgrades` and `PE:Arcane`
that the catalogue uses, computed by E-9.13 at every rank and formatted to the display precision of that line, MUST
equal the PE text within one display unit; a failure means the item belongs in X-RANK.

Who stores what: `04` stores `x_max` and the condition model of every effect; this document defines `x(r)` and X-RANK.
Riven values at rank r use E-4.18 of `04` with its own `(r + 1)` factor.

---

## 6. Capacity, polarity and Forma

### 6.1 Polarities and Forma types

Polarities: Madurai (M), Naramon (N), Vazarin (V), Zenurik (Z), Unairu (U), Penjaga (P), Umbra (UMB), Any (ANY), and
none (unpolarized). Data strings:

- slot polarities (`M:Weapons.Polarities`, `ExilusPolarity`, `StancePolarity`; `Warframes_data` `AuraPolarity`):
  `Aura` means ANY (the wiki's `{{Pol|Aura}}` symbol is "Any", `W:Polarity` L17; Vinquibus, Jade); `Universal`
  means none (Dual Heat Swords has `StancePolarity` `Universal` and "is the only melee weapon without a stance
  polarity", `W:Dual_Heat_Swords` L29; the same reading for Haalvu's `ExilusPolarity` `Universal` is UNVERIFIED,
  OQ-922); a missing field, `None`, `'None'` or an empty string means none (an unpolarized slot);
- mod polarities: wiki `Universal`, `None` and PE `AP_UNIVERSAL` mean "no polarity"; PE `AP_ATTACK` M, `AP_TACTIC` N,
  `AP_DEFENSE` V, `AP_POWER` Z, `AP_WARD` U, `AP_PRECEPT` P, `AP_UMBRA` UMB, `AP_ANY` ANY.

Sources: `W:Polarity` L8-18; `F:mods` §2.3; `DS:pe` §1.3. Weapon mods only use M, N, V, Z, U, P and UMB (Sacrificial
Steel and Pressure); unveiled rivens are M, N or V (`W:Riven_Mods` L40).

| Forma type | Applies | Restrictions | Source |
|---|---|---|---|
| Forma | any standard polarity (M, N, V, Z, U, P) or none, to any slot including pre-polarized and the exilus slot | unlimited count; resets rank (irrelevant) | `W:Polarity` L24-38; `W:Aura` L313-314 |
| Umbra Forma | UMB | only warframes and melee weapons (and Deconstructor); not directly on a warframe's exilus or aura slot | `W:Umbra_Forma` L2-4, L66-69 |
| Omni Forma | ANY on any slot except the stance slot | – | `W:Polarity` L17-18, L53-54 |
| Stance Forma | ANY on the stance slot | melee only | `W:Polarity` L17-18 |
| Aura Forma | ANY on the aura slot | warframe | `W:Aura` L22, L349 |

### 6.2 Drain after polarity

```text
E-9.14 Capacity charged by a general or exilus slot ω holding mod m at rank r (d = ξ(r) ≥ 0)
  c_ω = d                       if π_ω = none
  c_ω = ceil(d / 2)             if π_ω = π_m, or π_ω = ANY ∧ π_m ∉ {UMB, unknown}             (matching)
  c_ω = floor(1.25·d + 0.5)     otherwise (mismatch)                                          (Decision, UNVERIFIED)
  alternative (not used): ceil(1.25·d)        differs only for d ≡ 1 (mod 4): 1, 5, 9, 13, 17
  a non-Umbra mod in an UMB slot and an Umbra mod in an ANY slot are mismatches (Decision, UNVERIFIED, OQ-903)
  a riven uses its own polarity and drain (section 12.4); a riven whose polarity is not given has polarity
  `unknown`, which matches no slot (Decision)
```

Sources: `W:Mod` L162-164 (matching "reduces drain by half, rounded up"; non-matching "increases drain by a quarter,
rounded mathematically: drain of 0-1 by 0, 2-5 by 1, 6-9 by 2, 10-13 by 3, 14-16 by 4"); `W:Polarity` L20 ("50 %,
rounding up"; "25 %, rounded up"); `F:mods` §2.3. The Mod page wins because it gives an explicit bracket table
(per-mechanic page with a numeric table beats prose, overview section 1.4); EC-09-01, OQ-901.

### 6.3 Aura and stance capacity bonus

```text
E-9.15 Capacity added by a stance (weapon) or aura (warframe) slot holding mod m at rank r, b = −ξ(r) > 0
  β_ω = 2·b                     if π_ω = π_m or π_ω = ANY                                     (matching)
  β_ω = b                       if π_ω = none
  β_ω = floor(0.8·b)            otherwise (mismatch)                                          (Decision, UNVERIFIED)
  alternative (not used): floor(0.75·b + 0.5)    differs for b = 1, 2, 6
  β = Σ β_ω over the stance slot (weapon) or the aura slots (warframe)
  exalted melee: the X-STANCE stance (b = 5 at its fixed rank 3, Zenurik) in its fixed Zenurik slot gives
                 β = 2·5 = 10 (since U38.5)
```

Sources: `W:Aura` L20 ("80 % of listed drain, rounded down (… 5 generates 4, 9 generates 7)"); `W:Polarity` L22
("reduce it by 20 %"); `W:Mod` L166-167 ("reduces it by 25 %, rounded mathematically"); `W:Stance` L709-710 (5 maxed,
10 matching); `W:Exalted_Weapon` L205-210 (U38.5.2: exalted stances give capacity, stance polarity cannot be swapped),
L280 (U38.5: Zenurik polarity on exalted stances, +10). Two of three pages say 20 % and the Aura page gives worked
numbers; every maxed stance (b = 5) gets 4 under all three readings (EC-09-02, OQ-902).

### 6.4 Item capacity

```text
E-9.16 Item capacity                                                       key: cap.rank      severity: illegal
  C_cap = capacity_override                                      if the user entered one
  C_cap = rank · (1 + supercharged)                              otherwise
    rank ∈ [0, MaxRank]; MaxRank = W.MaxRank (30 or 40), warframe 30; default rank = MaxRank
    supercharged = Orokin Catalyst (weapons, incl. archguns, archmelee, robotic weapons) or Orokin Reactor (warframe);
                   exalted weapons: always true
  overlevel: MaxRank = 40 for the 55 records with M:Weapons.MaxRank = 40 (Kuva, Tenet, Coda weapons, Paracesis,
             Thermian RPG, Kuva archguns); rank 30 + 2·n_F needs at least n_F Forma applied (n_F ≤ 5)
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
  weapon:   U = Σ_{ω ∈ gen0..gen7, exilus} c_ω − β_stance
  warframe: U = Σ_{ω ∈ fgen0..fgen7, fexilus} c_ω − Σ β_aura
E-9.18 Capacity rule                                                   key: cap.exceeded   severity: illegal
  weapon: U ≤ C_cap under cap_mode (toggle T-35):
    ignore     no check (the inspiration's behaviour; D-092 compatibility profile)
    current    the build's polarity layout, slot positions free (Swap Polarity, E-9.20): ∃ assignment with N_forma = 0
    any_forma  (default) ∃ layout reachable with any number of Forma: N_forma* of E-9.19 exists
    budget(N)  N_forma* ≤ N
  warframe: checked only when frame_complete = true (part of T-35; the tool models only the damage-relevant frame
            mods; Decision)
```

`any_forma` is the default because it is the game's own limit: on a weapon without a stance slot, even with every slot
matched, 8 Galvanized-class mods of drain 16 need `8 · 8 = 64 > 60` (TV-09-03), so capacity can bind without regard
to Forma (on a melee weapon a maxed matching stance adds 10 and the same mods fit: U = 54).

Order of operations and rounding (all integer arithmetic): (1) `ξ(r)` per mod (E-9.12; a riven's drain per section
12.4); (2) per slot, `c_ω` or `β_ω` with the rounding stated in E-9.14 and E-9.15, each slot rounded on its own; (3) the
sums of E-9.17; (4) the comparison `U ≤ C_cap`. No other rounding occurs, and the order of slots does not matter.
Elsewhere in this document: per-rank effect values are f64 and never rounded to the display (E-9.13); riven ranges
are binary32 (E-9.56, overview NP-2 note on riven values); valence values are integers in tenths (E-9.70, E-9.71).

### 6.6 Minimum Forma

```text
E-9.19 Minimum Forma                                                        (report; constraint in modes any_forma, budget)
  Given: general mods (d_i, π_i) i = 1..n_m ≤ 8 (d_i = ξ(r) of mod i); innate general polarities I (multiset of 8,
         unfilled entries = none); exilus mod (d_x, π_x) and innate exilus polarity; stance mod (b, π_st) and innate
         stance polarity; C_cap.
  A layout assigns the n_m mods to distinct general slots (positions free, E-9.20) and decides per occupied slot:
    keep:  the slot keeps its innate polarity p, cost c = E-9.14(d_i, π_i, p), 0 Forma
    forma: the slot gets the mod's polarity, cost ceil(d_i/2), 1 Forma (0 if p already matches);
           if π_i = UMB and Umbra Forma is not allowed on the item (6.1), or π_i = unknown (a riven without a given
           polarity): the slot gets none instead, cost d_i, 1 Forma
  The exilus and stance slots have the same two options (stance: β = 2b with forma; Stance Forma only for ANY;
  no option for a stance_fixed weapon, whose fixed Zenurik stance always gives 2b).
  Warframe (E-9.20): the fexilus polarity joins the multiset, so I has 9 entries and the exilus mod is assigned to
  one of them like a general mod; the aura slots keep their polarities and have the keep/forma options (Aura Forma
  only for ANY).
  Unoccupied slots keep their innate polarity.
  N_forma* = min N_forma over layouts with U ≤ C_cap; ties broken by the smaller U. No layout ⇒ cap.exceeded in every
  mode but ignore.
  Algorithm (exact): dynamic programming over the mods with state = remaining count of innate general slots per
  polarity value (at most 2^8 states, 2^9 for the warframe) and value = min U per Forma count; then the 2 × 2 exilus
  and stance options (weapon) or the aura options (warframe).
  Report: N_forma* by Forma type (Forma, Umbra Forma); for an overlevelled weapon at rank > 30 also the lower bound
  ceil((rank − 30)/2) on Forma applied (E-9.16).
E-9.20 Slot positions                                                       (Decision)
  General-slot polarities are treated as a multiset: once a weapon has been polarized it can rearrange its
  polarities with Swap Polarity for free (W:Polarity L36-37); the data lists innate polarities without positions
  (M:Weapons.Polarities).
  Warframe exilus slot: swappable with the general slots (W:Umbra_Forma L68: an Umbra polarity "can be swapped to" a
  Warframe's exilus or aura slot from other slots; UNVERIFIED, OQ-911).
  Aura slots: not swappable (W:Aura L352, U15.1 patch note, precedence 2; contradicted for aura by the later
  W:Umbra_Forma L68 note, UNVERIFIED, OQ-911).
  Weapon exilus and stance slots: not swappable (no source; UNVERIFIED, OQ-911). Exalted stance slots: not swappable
  (W:Exalted_Weapon L208, U38.5.2).
```

For a fixed slot assignment, the cheapest way to reach U ≤ C_cap is to Forma the slots with the largest saving
`keep − ceil(d/2)` first, which is how the test vectors were computed by brute force over all assignments (TV-09-02,
TV-09-03, TV-09-17). Mod order in the general slots (which drives element combination, `01`) is independent of the
polarity layout under E-9.20.

Vinquibus: the primary and melee records share one Forma layout (`W:Umbra_Forma` L67; EC-09-05). A build of either
form stores the shared layout; the UI shows and edits it once for both forms.

---

## 7. Exilus slot

```text
E-9.21 Exilus                                             keys and severities of E-9.2 (slot.count), E-9.3, E-9.8
  exists: has_exilus(W) (E-9.2); warframe always
  content: E-9.3 (isUtility) ∧ E-9.8
  innate polarity: W.ExilusPolarity, read per section 6.1 (M:Weapons: Madurai or Naramon on 331 of the 356 primary
                   and secondary records; missing, 'None', '' or 'Universal' on the other 25, for example Latron,
                   Aeolak, Phage, Angstrum, Riot-848 and the exalted weapons, which therefore have an unpolarized exilus
                   slot; missing on 249 of 250 melee records; Madurai on the 12 kitgun chamber records);
                   polarizable with Forma
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
             archmelee, robotic → none;  Arquebex, Ironbride (Necramech exalted) → none (Decision, UNVERIFIED, OQ-906)
             exalted and pseudo-exalted weapons, Garuda Talons → as their slot class (sourced, U38.5)
             Dex Pixia (a Secondary record) → secondary (UNVERIFIED, OQ-906)
             warframe → farcane0, farcane1 ∈ warframe
  a filled arcane slot that does not exist: slot.count (E-9.2); ranks: E-9.11; duplicates: E-9.10 D2
  Bow, Shotgun and Akimbo arcanes are legal in any slot of their family; their effects are limited to that weapon
  type (04) (Decision, UNVERIFIED, OQ-914)
E-9.30a Arcane gating by weapon tags                                        key: arcane.gate   severity: illegal
  an arcane a with gate(a) ∩ tags(W) ≠ ∅ is illegal on W;  gate(Melee Crescendo) = {NO_FINISHER}
  (Whipclaw, Shattered Lash, Landslide Fists and their Primes); no other arcane has a gate
```

Sources: `W:Arcane_Enhancement` L19-26 (one arcane per weapon; kitgun and zaw arcanes in a separate slot; archguns one
primary and one secondary; sentinel weapons none; archmelee and Necramechs none; no duplicates on one item), L1360-1366,
L1548-1554, L1755-1761 (adapters; archgun slots; separate from Pax and Exodia slots), L1940 (primary passives do not
apply to secondaries); `M:Arcane` type counts; `W:Exalted_Weapon` L233-241 (U38.5 patch note, precedence 2: "the
addition of Arcane slots to all Exalted weapons!"; Garuda's Talons get one "nonetheless"), L312 (the new exalted
weapons of Atlas, Gara, Khora and Ash "will receive the same treatment"), L121 (U44.0.3 overlapping arcane and exilus
slots on exalted primaries and secondaries), L145-146 (U39.1 fixes; L146 fixes an exalted melee arcane slot that
appeared unlocked in the Navigation loadout, which concerns the adapter, assumed installed in section 2.2),
L178-180 (U38.5.3: Melee Crescendo removed from Whipclaw, Shattered Lash and Landslide Fists "as they cannot perform
Finishers"); `W:Primary_Merciless` L34. Which warframe arcanes act on an archgun (class-restricted ones such as Arcane
Precision do not, `W:Archgun`) is an effect rule of `04` §5.5. The Arcane Helmet case (one warframe arcane only) is
out of scope (EC-09-35).

Tektolyst Artifact arcanes sit on operator gear, which this tool does not model, but one of them acts on weapons:
Zid-An Uskos grants up to +2.50 non-combining Heat to secondary weapons (exalted secondaries included since U41.0.3)
on operator and Tauron Strike kills (`W:Zid-An_Uskos` L2, rank table, Notes, patch history). It is not a slot of this
document; until `04` catalogues it as an external buff (OQ-925) a user enters it with the custom stat `heatAdd`
(T-26, a non-combining Heat addition, overview section 6.3).

---

## 9. Archon shard slots

```text
E-9.31 Archon shards                                                                          severity: illegal
  key shard.count: at most 5 shards on the warframe
  key shard.bonus: each shard z: colour_z ∈ {Crimson, Amber, Azure, Topaz, Violet, Emerald}, tau_z ∈ {false, true},
                bonus_z ∈ bonuses(colour_z)                                   (exactly one bonus per shard)
  duplicates: allowed (same colour and same bonus on several shards)
  N_cav = #{z : colour_z ∈ {Crimson, Azure, Violet}}          (input of 04 E-4.12, whatever bonus each shard carries)
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

Applies only to buffs whose caster is the user's own warframe (`04` §4.4 `buff_str`, catalogue field `caster =
self`). Ally-cast buffs are never restricted. The caster of T-13 (Roar) is T-37 (`self` or `ally`, default `ally`;
overview section 6.1): it is ally-cast, and so unrestricted, unless the user sets T-37 to `self`.

Only Roar reaches allies ("increasing the damage output of himself and his allies", `W:Roar` L14). Eclipse, Xata's
Whisper and Toxic Lash act on the caster's own weapons only ("Temporarily increase Mirage's weapon damage",
`W:Eclipse` L9; "Wield Void damage for all attacks from equipped weapons", `W:Xata's_Whisper` L8; "imbue all her
weapons", `W:Toxic_Lash` L14; `04` §5.6 "self only"), so T-14 … T-16 are always self-cast and E-9.32 always applies
to them (overview T-37, adopted from OQ-926; overview T-16 and D-037 already read Toxic Lash as cast by the user's
own Saryn). T-14 and T-15 together are therefore illegal whenever the frame identity is set (H3 on Mirage or Xaku, H1
on any other frame); with the frame identity unset only H1 and H5 are checked (below), as in TV-09-14 (f).

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
  slots(S-count) = warframe: fgen0..fgen7, fexilus, aura0, aura1;  active weapon: gen0..gen7, exilus (the stance
                   slot holds no set mods);  plus the declared companion and robotic-weapon set mods
  pieces(S) = { identity(m) : set(m) = S, m in those slots, m ∉ set_noncounting }      (distinct identities)
  set_noncounting = { Mecha Empowered }   (the aura is a Mecha piece but gives no set bonus: W:Mecha_Empowered L42
                                            "does not provide a set bonus, despite what the arsenal UI shows";
                                            04 §9 Mecha row, OQ-439)
  N_S = |pieces(S)|,  0 ≤ N_S ≤ NumUpgradesInSet(S)
  active weapon item = the weapon of the evaluated attack (an exalted weapon when the attack is an exalted attack);
                       set mods on holstered weapons never count (U38.5)
  rank: N_S does not depend on the ranks of the pieces
  set(m) = PE modSet (M:Mods.Set as cross-check); NumUpgradesInSet from M:Mods.Sets
E-9.41 Declared companion pieces                                            key: set.companion  severity: illegal
  a declared piece must be a set mod a companion or robotic weapon can equip; melee robotic weapons (Deconstructor,
  Deconstructor Prime, Akaten, Batoten, Lacerten) cannot equip set mods; Hunter companion mods cannot go on sentinels
  (checked against companion_kind; with companion_kind = none no companion or robotic-weapon piece may be declared;
  a robotic-weapon piece needs companion_kind ∈ {sentinel, hound}, the companions with weapons of M:Weapons slot
  Robotic or Hound) (Decision for the companion_kind input)
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
                            ∨ W.Family = 'Garuda Talons'      (W:Exalted_Weapon L106-107: "This is also true for
                                                               Garuda Talons"; named list, Decision)
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
  riv.class = riv_class(W) ≠ none  ∧  riv.family = riv_family(W)
  riv_family(W) = X-FAMILY(W.Name)  if listed
                = W.Family          if present                              (M:Weapons.Family)
                = W.Name            otherwise                               (Decision; EC-09-72)
  X-FAMILY (hand-authored, multi-record weapons): Dark Split-Sword (Dual Swords), Dark Split-Sword (Heavy Blade)
           → 'Dark Split-Sword'
  kitgun: family = chamber name (Catchmoon (Primary) and (Secondary) share 'Catchmoon'); zaw: family = strike name
E-9.52 Count: at most one riven per weapon item (E-9.10 D4)                 key: dup.riven      severity: illegal_unverified
```

Sources: `W:Riven_Mods` L36-44 (classes; one weapon and its variants), L396-414 (veiled class rules: rifle rivens on
non-shotgun primaries, kitgun rivens on primaries or secondaries, robotic on any robotic weapon), L920 (robotic rivens
use the base values of their associated class), L1524-1526 (variants MK1, Prime, Vandal, Wraith, Dex, Prisma, Mara,
Syndicate share; Mutalist Cernos and Quanta, dual vs single sidearms, Dex Dakra vs Dakra Prime, Nikana vs Dragon
Nikana do not; no rivens for exalted weapons); `W:Kitgun` L112-127 (chamber-bound, both versions); `W:Zaw` L642
(strike-bound); `F:mods` §8.2. The wiki `Family` field reproduces every pair the page lists (Mutalist Cernos, Dex
Dakra, Dakra Prime, Dragon Nikana, Akbolto and Afuris are their own families; Rakta Cernos and Cernos Prime are family
Cernos; Kuva Hek and Vaykor Hek are family Hek), checked on `M:Weapons` for this document; that Kuva, Tenet and Coda
variants share the base weapon's riven is UNVERIFIED (OQ-912). Two gaps need the X-FAMILY list and the fallback: the
two records of Dark Split-Sword, one weapon with two forms, have different `Family` values (each record's own name;
both carry the Dual Swords and Heavy Blade stance tags, section 3.2), and `Family` is missing on Nunchasa, Aksondol,
Sagek Prime and Thermian RPG among the non-exalted guns (counted for this review; EC-09-71, EC-09-72).

### 12.2 Traits, layout and signs

A riven instance is `{class, family, rank r ∈ 0..8, polarity ∈ {M, N, V, unknown}, drain (section 12.4),
traits[1..n]}`; traits are in display order (top to bottom, which `01` reads for the element order) and each is
`(trait id, displayed value)`.

```text
E-9.53 Displayed value → v: the conversion table of 04 §8.2 (one owner). Parry Angle, which 04 §8.2 files under the
       percentage splices, shows no % sign on the card and is still v = disp / 100 ("a percentage despite not being
       shown as such", W:Riven_Mods L987, L1222).
E-9.53a Order of checks (each step uses only traits that passed the earlier ones)
  1. T1 and T2 of E-9.55; a trait failing T2 has no coefficient b, is reported and takes no further part
  2. roles and layout (E-9.54)
  3. T3 to T6 of E-9.55
  4. value range (E-9.56)
E-9.54 Roles and layout                                                     key: riven.layout   severity: illegal
  sgn_i = sign(v_i / b_i)          (b_i = coefficient of trait i for the riven's column; b < 0 for Weapon Recoil,
                                    Ammo Efficiency and Chance to Gain Combo Count, so a beneficial value of the first
                                    two has sgn = +1)
  n_pos = #{sgn_i = +1}, n_neg = #{sgn_i = −1};   require n_pos ∈ {2, 3} ∧ n_neg ∈ {0, 1}  (2 to 4 traits)
E-9.55 Trait rules                                              key: riven.trait    severity: illegal unless stated
  T1 trait ids are distinct
  T2 every trait has a coefficient for the riven's column (04 §8.3; "–" = cannot roll)
  T3 positive-only traits have sgn = +1: Heat, Cold, Electricity, Toxin, Punch Through, Additional Combo Count Chance,
     every spliced trait, and Melee Damage on the Melee and Zaw columns (and companion rivens on the Melee column)
     (W:Riven_Mods L850-851 with footnote 1 at L916; the melee range table has no negative Melee Damage range, L306);
     negative-only: Chance to Gain Combo Count has sgn = −1 (footnote 3)
     Initial Combo: positive-only with severity illegal_unverified (footnote 1 only in the coefficient table L1108;
     the base-value table L868 has none and the melee range table L312 lists negative ranges; EC-09-74, OQ-921)
  T4 Damage on the Rifle, Shotgun, Pistol/Kitgun and Archgun columns may have sgn = −1 (Decision, UNVERIFIED: the
     footnote "positive only" on the shared row L850-851 contradicts the negative ranges of the gun range tables
     L68, L127, L186, L245 and the Update 44.1 patch image of a Hikou riven with −222.6 % Damage; 04 OQ-424)
  T5 at most one spliced trait; the spliced trait must be available for the class (table below)
  T6 primary elements on one riven: at most 3 (follows from T3 and n_pos ≤ 3)
E-9.56 Value range                                                          key: riven.range    severity: illegal
  e_0, e_1 = E-4.18 (04) at the riven's rank r with quality u = 0 and u = 1 (04's factor W = 0.9f + 0.2f·u, written
             W_q here to keep W for the weapon record), the layout factor L from (n_pos, n_neg, sgn_i), D = 1.2,
             evaluated in binary32 exactly as E-4.18
  [v_lo, v_hi] = [min(e_0, e_1), max(e_0, e_1)]          (e_1 < e_0 whenever b·L < 0)
  legal: v_lo − tol ≤ v_i ≤ v_hi + tol, tol = one display unit of the trait (0.001 for percentage traits,
         0.01 for faction ×, 0.1 for m, s and combo counts), compared in f64      (Decision: covers rounding and
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
shotgun Zoom 2026-09-28). A spliced riven keeps the same `(n_pos, n_neg)` layout, so E-9.54 applies unchanged; a 3+1
spliced riven shows 4 traits (TV-09-11).

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

Rank: 0..8 (E-9.11). Polarity: M, N or V (`W:Riven_Mods` L40). MR 8-16: irrelevant. The element order of a riven's
traits (bottom-up) is `01`'s.

```text
E-9.59 Riven drain and polarity                                             (inputs; no rule key)
  input: the drain d_u the card shows at the riven's rank r_u, and its polarity
  ξ0 = d_u − r_u;   ξ(r) = ξ0 + r at any evaluated rank r (E-9.12 with baseDrain = ξ0)   (Decision, UNVERIFIED, OQ-905)
  drain not given (inspiration links carry none): ξ(8) = 18, so ξ0 = 10, the top of the range F:mods §8.2 reports
                  ("commonly 10–18 at rank 8"; a conservative choice, Decision, UNVERIFIED, OQ-924)
  polarity not given: unknown, which matches no slot (E-9.14) and cannot be matched by a Forma (E-9.19)
  the capacity report names every riven whose drain or polarity is a default
```

No source gives riven drain: PE's `baseDrain 9`, `fusionLimit 0` belongs to the veiled placeholder (`DS:pe` §1.3),
and the wiki has no drain field. The `+1 per rank` growth of E-9.59 is the rule of every other mod (E-9.12) and is
UNVERIFIED for rivens. The defaults make `cap.exceeded` computable under the default mode `any_forma` for every
imported build: a riven with unknown polarity and drain 18 costs 18 in an unpolarized slot and 23 in a polarized one
(TV-09-18).

---

## 13. Incarnon evolutions

```text
E-9.60 Incarnon availability and choices                                                      severity: illegal
  line(W) = the hand-authored perk table row for the exact weapon record W (native Incarnon weapons and every
            variant that accepts its family's Genesis); undefined for Akimbo and Dual variants
  key incarnon.line: a perk or genesis_installed on a weapon without line(W)
  T(W) = 5 evolutions (native: Felarx, Innodem, Laetum, Phenmor, Praedos, Onos, Ruvox, Thalys) or 4 (Genesis)
  evolution I: the Incarnon form (no choice); available iff line(W) defined ∧ (native ∨ genesis_installed)
  key incarnon.tier: evolution t = 2..T(W): inc_choice[t] ∈ perks(W, t) ∪ {none} (default none);
                    |inc_choice[t]| ≤ 1; no choice for t > T(W)
  key incarnon.form: a perk choice requires evolution I to be available
  value(perk, W) = the perk's value in W's variant column (Furis X = 28, Mk1-Furis X = 34)
  the Incarnon-form attacks of the data (M:Weapons attacks named "Incarnon …") are evaluable only when evolution I
  is available
```

Sources: `W:Incarnon` L11 (all sub-types such as Prime, Wraith, Vandal accept the Genesis; Akimbo and Dual weapons do
not), L180-188 ("Each weapon has a total of 4 or 5 evolutions"; "The remaining Evolutions provide 2 or 3 different
selectable perks … Only one perk can be selected per Evolution, which can be changed freely"), L228 (Genesis weapons
"only have 4 Evolutions in total"; challenges are cleared once per weapon type); `W:Furis_Incarnon_Genesis` (cached
under `wiki-modules/`) evolution table: EVO2 Haven Foray, Stormburst (X = 28 / 34); EVO3 Practiced Grip, Extended
Volley, Executioner's Fortune; EVO4 Headcracker, Prelude of Might, Elemental Balance (X = 28 % / 39 %); `F:mods` §9;
`I:model` §2.3 (the inspiration stores tiers 18-22 with 18 = form). Perk effects, their attack filters and conditions
are `04` §5.12; the perk table is hand-authored because no data source has it (`DS:wiki` §3, `DS:wfcd` §2.5, `DS:pe`
§1.6). The WFCD `Incarnon` tag marks only 8 weapons and must not be used to detect Incarnon capability (`DS:wfcd`
§2.5).

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
     severity illegal on Kuva and Tenet weapons; illegal_unverified on Coda weapons (the U36 snap note names only
     Kuva and Tenet weapons; the page body states the snap for fusion results of all three; OQ-909)
  progenitor_frac = v10 / 1000                (overview symbol v)
E-9.71 Valence fusion (helper for the UI; integer arithmetic in tenths)
  fuse(a10, b10) = 600 if f ≥ 580 else f,   f = min(600, floor(11 · max(a10, b10) / 10))
  the element of the result is the player's choice of the two (U30.5)
```

Sources: `W:Kuva_Weapons` L96 (seven elements, 25-60 % of base damage), L98 (Elemental Vice at rank 40 switches the
element freely); `W:Tenet_Weapons` L116; `W:Coda_Weapons` L145; `W:Valence_Fusion` L13-21 (×1.1 of the higher bonus,
rounded down to the first decimal, capped at 60 %; ≥ 58 % rounds up to 60 %), L59-85 (fusions needed: 52.8 → 1,
48.0 → 2, 43.7 → 3, 39.8 → 4, 36.2 → 5, 33.0 → 6, 30.0 → 7, 27.3 → 8, 25.0 → 9; reproduced exactly by E-9.71),
L89 (reports of hidden precision), L96-99 (U36 58 % rule: "Kuva and Tenet weapons with 58% Elemental Bonus or above";
Coda weapons came later and are not named), L106-112 (U30.5 element choice); `F:mods` §10;
`F:core` §3.2. The roll distribution (uniform since U42 for Kuva and Tenet, `F:mods` §10) does not affect legality.

---

## 15. Worked examples

All arithmetic was checked with `python3` (throwaway scripts in the session scratchpad); riven ranges use numpy
float32 step by step as E-4.18 requires.

**Example 1, capacity and Forma (TV-09-02).** Braton Prime, rank 30, Catalyst: C_cap = 60. `M:Weapons`: no innate
general polarities, exilus Naramon. Mods at maximum rank, drain by E-9.12 from `PE:Upgrades` (`baseDrain` +
`fusionLimit`): Serration 4 + 10 = 14 (M), Galvanized Chamber 6 + 10 = 16 (M), Primed Bane of Grineer 4 + 10 = 14 (M),
Point Strike 4 + 5 = 9 (M), Vital Sense 4 + 5 = 9 (M), Hellfire 6 + 5 = 11 (N), Infected Clip 6 + 5 = 11 (N), Hunter
Munitions 4 + 5 = 9 (M). Unpolarized U = 93 > 60. Savings per Forma `d − ceil(d/2)`, in the same mod order: 7, 8, 7,
4, 4, 5, 5, 4. The five largest give 32 (U = 61 > 60); six give 36 (Galvanized Chamber, Serration, Primed Bane,
Hellfire, Infected Clip and one of the 9-drain mods): U = 57 ≤ 60. N_forma* = 6, U = 57. Placing Hellfire in a Madurai
slot instead of its own Naramon one costs floor(1.25 · 11 + 0.5) = 14 (ceil(13.75) = 14 under the alternative rule
too) instead of 6, and U = 65 > 60. `F:mods` §2.4 computes the same build with the stale `M:Mods` drains (Point Strike
and Vital Sense 7, Infected Clip 9: 87, 4 Forma, U = 60); PE wins (EC-09-73).

**Example 2, riven validation (TV-09-10).** A Braton riven (class Rifle, family Braton) at rank 8 shows Critical
Chance +220.0 %, Multishot +130.0 %, Damage −95.0 %. Roles: b > 0 for all three, so sgn = (+1, +1, −1), layout 2+1,
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
| EC-09-01 | Non-matching drain: `floor(1.25d + 0.5)` (Mod page) vs `ceil(1.25d)` (Polarity page); they differ for d = 1, 5, 9, 13, 17 (Point Strike at rank 5, d = 9: 11 vs 12). Engine: Mod page; UNVERIFIED. | E-9.14; `W:Mod` L164; `W:Polarity` L20; OQ-901 |
| EC-09-02 | Aura/stance mismatch bonus: 80 % floor (Aura), 75 % rounded (Mod), −20 % (Polarity); a maxed stance (5) gives 4 under all; differs for b = 1, 2, 6. Engine: floor(0.8b). | E-9.15; OQ-902 |
| EC-09-03 | ANY matches every polarity except Umbra (sourced); an Umbra mod in an ANY slot and a non-Umbra mod in an Umbra slot are treated as mismatches rather than neutral (both UNVERIFIED: the page only says ANY does not match Umbra). | `W:Polarity` L17; `F:mods` §2.3; OQ-903 |
| EC-09-04 | Umbra Forma only on warframes, melee weapons and Deconstructor (Prime); not directly on a warframe exilus or aura slot; Umbra mods elsewhere can never be halved. | `W:Umbra_Forma` L66-69 |
| EC-09-05 | Vinquibus primary and melee forms share one Forma layout; Umbra Forma applied through the melee form does not halve anything on the primary. | `W:Umbra_Forma` L67; section 6.6 |
| EC-09-06 | Overlevelled weapons (Kuva, Tenet, Coda, Paracesis, Thermian RPG, Kuva Ayanga and Grattler): rank 40 after 5 Forma; capacity 40, 80 with Catalyst. | E-9.16; `W:Weapons` L21-27 |
| EC-09-07 | Exalted weapons: Catalyst pre-installed; exalted melee stance fixed in a Zenurik slot, +10 capacity since U38.5 (b = 5 from X-STANCE, the data says otherwise: EC-09-67), polarity not swappable since U38.5.2; pseudo-exalted melee have no exilus slot. | E-9.2, E-9.3, E-9.15; `W:Exalted_Weapon` L10, L94-96, L205-210, L280 |
| EC-09-08 | Garuda Talons: Madurai stance polarity, any claw stance (data class Claws, not exalted). | `W:Exalted_Weapon` L95 |
| EC-09-09 | Melee exilus slot since U35 (after Whispers in the Walls); `W:Exilus_Mods` L5 still says primary and secondary only (outdated). | E-9.21 |
| EC-09-10 | Exilus flag from PE `isUtility`; wiki `IsExilus` differs on 7 mods: Flawed Ravage flagged by the wiki only (likely data error, UNVERIFIED: a crit-damage mod; `F:mods` §12, edge case 31); Primed Rifle/Shotgun Ammo Mutation, Flawed Whirlwind and three warframe mods flagged by PE only. | E-9.3; measured on `PE:Upgrades` × `M:Mods` |
| EC-09-11 | `CompatibilityTags` missing on about 72 guns (74 primary and secondary records counted for this document, mostly shotguns, pistols and dual pistols, plus the rifle Higasa) and `Polarities` missing on 74 guns: read as empty; the data report lists them. | E-9.5; `DS:wiki` §3, §11; OQ-917 |
| EC-09-12 | `SEMI_AUTO` is on only 12 weapon records (Latron has none): derived from triggers. | E-9.5, E-9.6 |
| EC-09-13 | Cannonade needs every trigger component Semi-Auto: Argonak ("Auto / Semi"), Nagantaka ("Semi / Mag Burst"), EFV-8 Mars excluded and Latron, Vectis, Strun, Bronco, Lato, Lex Incarnon allowed (named on the pages); Laetum, Vasto (Incarnon Burst), Torid (Incarnon Held), Dual Toxocyst (Incarnon Auto) excluded by derivation from `M:Weapons` triggers; missing per-attack triggers inherit the weapon trigger; a Genesis weapon without its Genesis is judged without the form (EC-09-79). | E-9.6 |
| EC-09-14 | Duplex (Tigris, Sancti Tigris, Tigris Prime, Zylok, Zylok Prime) is not Semi-Auto for Cannonade (Decision). | `W:Trigger_Type` L20; OQ-907 |
| EC-09-15 | Kitguns: primary kitguns cannot take Semi-Rifle/Shotgun Cannonade (MODULAR_GUN); secondary kitguns can (U37); MODULAR_GUN is missing on the Gaze, Rattleguts and Vermisplicer records and is derived for every kitgun. | E-9.5, E-9.6 |
| EC-09-16 | Synth Charge: base magazine ≥ 6 (Kulstar 3 allowed by a bug); not on SINGLESHOT or exalted weapons; raising the magazine of a weapon below 6 does not qualify it; equippable on the Burst Laser. Effect rules are `06` EC-06-48. | E-9.7 |
| EC-09-17 | Split Flights excluded from Lenz, Kuva Bramma, exalted weapons, Proboscis Cernos and Nataruk by tags; it excludes Split Chamber and Galvanized Chamber. | E-9.5, E-9.9 |
| EC-09-18 | Vaykor Hek cannot equip Scattered Justice (`SYNDICATEMODBLOCKED`). | E-9.5; `M:Mods`, `M:Weapons` |
| EC-09-19 | Non-node mod `Type` strings (`Rifle (No AoE)`, `Pistol (No AoE)`, `{Pistol}`, `Unique`, `Rifle or Pistol`, `Robotic Weapon`) are normalised; `Claws` is the beast-claw node. | E-9.4 |
| EC-09-20 | Multi-tag `CompatibilityTags` mean "any of" (Sniper Ammo Mutation would be unusable otherwise). | E-9.5; OQ-913 |
| EC-09-21 | Stance tag inconsistencies: Harrowing Spire `BAYONET` vs weapon `BAYONET_STANCE`; one weapon `SCYTHE_STANCE` vs mods `SCYTHES_STANCE`; `CLAWS_STANCE_STANCE` and `HEAVY SCYTHE_STANCE` are consistent on both sides. | E-9.5 |
| EC-09-22 | `Incompatible` lists: 3 asymmetric entries (Primed Electrified Barrel; Primed Streamline twice); union of both directions; no transitive closure. | E-9.9 |
| EC-09-23 | Stacking counter-examples (Charged + Primed Chamber; Heavy Caliber + Serration; Spoiled Strike + Pressure Point; Vigilante Armaments + Split Chamber; Augur Pact + Hornet Strike) vs non-obvious exclusions (Critical Delay ⟂ Point Strike; Critical Deceleration ⟂ Blunderbuss; Berserker Fury ⟂ Fury; Argon Scope ⟂ Galvanized Scope; Concealed Explosives ⟂ Energizing Shot). | E-9.9; `W:Mod/Compatibility` L262-266 (Critical Deceleration, Berserker Fury); `W:Mod` L272-294 "Nonstandard Mods" (Critical Delay, Split Flights, Charged + Primed Chamber); `F:mods` §5.1 (the other examples); `M:Mods.Incompatible` (Concealed Explosives) |
| EC-09-24 | Mod identity is the PE `uniqueName`; Beginner (Flawed) and Intermediate copies share the plain name in PE, Primed (Expert) copies too in some records; only identities present in `M:Mods` are offered. | E-9.10 |
| EC-09-25 | Per-rank exceptions are the X-RANK table: Internal Bleeding, Hemorrhage, Magnetic Welt `0.10 + 0.05r` (not `0.35(r+1)/6`); Critical Precision, Clip Delegation, Necrophagic Vigor, Dreadful Killshot and the affine arcanes (Melee Duplicate, Primary Bulwark, Primary Overcharge, Cascadia Empowered, Primary Debilitate …) are damage-relevant; ammo mutations and other affine lines without damage effect are not listed; durations and stack caps follow the PE series where they vary (Double Tap, Gorgon Frenzy, Primary Plated Round); PE display rounding is irregular, values come from the maximum. | E-9.13; `PE:Upgrades`, `PE:Arcane` |
| EC-09-26 | Sacrificial Steel/Pressure Sentient bonus is linear per rank in PE (×1.03 … ×1.33, step 0.03); the wiki rank table that looked non-linear (`F:mods` §3) is superseded. Set-level values are `04`'s (OQ-415 there). | E-9.13; `PE:Upgrades` |
| EC-09-27 | Rank-gated arcane lines: Merciless reload, Deadhead headshot multiplier and recoil, Dexterity combo duration only at rank 5; "+1 Arcane Revive" from rank 3 (Arcane Crepuscular, Avenger …); arcane maximum rank 3 for Kitgun and Zaw arcanes. | E-9.11, E-9.13; `PE:Arcane` |
| EC-09-28 | Faction mods scale the bonus part: Bane ×1.05 … ×1.30, Primed ×1.05 … ×1.55, Flawed ×1.025 (shown 1.03) … ×1.10. | E-9.13 |
| EC-09-29 | Set bonus: distinct pieces only ("require different Mods", `W:Set_Mods` L406), active weapon only since U38.5, exalted weapons count since U38.5, rank irrelevant, one piece already gives the 1-piece bonus. | E-9.40 |
| EC-09-30 | Robotic weapon set mods count; the same mod on the player's weapon and the robotic weapon counts once; melee robotic weapons cannot hold set mods; Hunter companion mods not on sentinels. | E-9.40, E-9.41 |
| EC-09-31 | U41 fixed eight set bonuses; data or tests from before U41 may show broken behaviour. | `W:Set_Mods` L229-232 |
| EC-09-32 | Archguns: 2 arcane slots (primary, secondary) since U40; class-restricted warframe arcanes do not act on archguns (effect, `04`); sentinel weapons, archmelee and Necramechs have no arcane slot. | E-9.30 |
| EC-09-33 | Exalted weapons (pseudo-exalted included) and Garuda Talons have an arcane slot of their slot class since U38.5 ("the addition of Arcane slots to all Exalted weapons"); only Dex Pixia's family and the Necramech exalted weapons remain open. | E-9.30; `W:Exalted_Weapon` L233-241, L312; OQ-906 |
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
| EC-09-44 | Riven Damage: the "positive only" footnote on the shared Damage / Melee Damage row vs negative ranges in the gun range tables (L68, L127, L186, L245) and a Hikou (pistol) patch image: negative allowed on gun columns (UNVERIFIED); Melee Damage has no negative range (L306) and stays positive-only (sourced). | E-9.55 T3, T4; `04` OQ-424 |
| EC-09-45 | Physical riven traits need > 25 % of that type "usually"; only a warning (exceptions exist). | E-9.57; OQ-910 |
| EC-09-46 | Splice: at most one spliced trait; always positive; the layout stays (the replacement random trait is negative if a consumed trait was); the spliced trait keeps the larger random value, a negative's included; a spliced trait can coexist with its own components (Viral + Cold + Toxin). | E-9.55 T5; `W:Riven_Mods` L1298-1310 |
| EC-09-47 | Shotgun rivens can roll Zoom since 44.0.2 (needed for the Weak Point splices). | `F:mods` §8.1 |
| EC-09-48 | Riven element order (bottom-up), Acuity multishot lock and the multishot floor are not build rules (`01`, `02`). | out of scope |
| EC-09-49 | Riven ranges scale with `(r + 1)`; a rank-0 Rifle 2+1 Critical Chance is 22.27 % … 27.22 %. | E-9.56 |
| EC-09-50 | Valence: ≥ 58 % snaps to 60 % (U36, Kuva and Tenet), so 58.0 … 59.9 % is illegal there and `illegal_unverified` on Coda weapons (not named by U36); fusion keeps the higher value; the player picks the element (U30.5); hidden precision (a 52.7 % weapon reported fusing to 60 %). | E-9.70, E-9.71; OQ-909 |
| EC-09-51 | `IsLichWeapon` is missing on Kuva Ghoulsaw; adversary weapons are recognised by `Traits`. | E-9.70 |
| EC-09-52 | Incarnon: 4 evolutions for Genesis, 5 for native; one perk per evolution, changeable; no Genesis for Akimbo/Dual variants; perk values per variant (Furis 28 vs Mk1-Furis 34; Elemental Balance 28 % vs 39 %). | E-9.60 |
| EC-09-53 | Incarnon perk effects (magazine perks not on the form, Torid Plentiful Mayhem, Braton GunCO bug) are effect rules of `04`/`06`. | out of scope |
| EC-09-54 | Ammo Efficiency cap 0.75 (Hotfix 44.1.3) is an effect clamp of `06`; a riven Ammo Efficiency splice is legal at any in-range value. | out of scope |
| EC-09-55 | Minimum capacity of low-rank items: `15 + 1 per 2 MR (+1 per Legendary Rank)` (Mod page) vs `= MR up to 30` (Polarity page) vs `max(15 + rank/2, MR)` (Weapons page); ignored, `capacity_override` instead. | E-9.16; OQ-908 |
| EC-09-56 | Swap Polarity needs a weapon polarized at least once; for a never-polarized weapon whose optimal layout permutes innate polarities, the report notes that 1 Forma may be needed to unlock Swap Polarity. | E-9.20; `W:Polarity` L36 |
| EC-09-57 | Warframe aura slot polarity cannot be swapped with other slots; Jade has two aura slots. | `W:Aura` L352; `W:Mod` L143 |
| EC-09-58 | Catalyst doubles the overlevel ranks too; it cannot be removed and survives Forma. | `W:Weapons` L27; `W:Orokin_Catalyst` L52, L61 |
| EC-09-59 | `M:Mods` puts `POWER_WEAPON` in the incompatibility tags of 62 mods (weapon augments, Synth Charge, Split Flights …); 16 of them are lifted for exalted weapons by X-EXALT (U38.5, U38.6; Harkonar Scope for Neutralizer only), so 46 still exclude every exalted weapon. | E-9.5; EC-09-68 |
| EC-09-60 | Dex Pixia (Razorwing) is a `Secondary` record in the data and is treated as a secondary; Regulators count as secondaries for arcanes and shards. | E-9.1; `F:weapon` §14 |
| EC-09-61 | Arquebex and Ironbride take Archgun and Archmelee mods; they get no arcane slots (Necramechs have none; Decision against "all Exalted weapons", OQ-906); no rivens (exalted by `Class`, EC-09-80). | E-9.4, E-9.30, E-9.50; `W:Mod/Compatibility` L199-201 |
| EC-09-62 | Robotic, hound and archwing weapons have no exilus slot; robotic weapons no stance slot. | E-9.2; `W:Mod` L146 |
| EC-09-63 | The riven faction trait list of the inspiration lacks Orokin (D-408 of `04`). | `04` D-408 |
| EC-09-64 | Galvanized stack rules, Galvanized Shot's 3 stacks / 14 s and Galvanized Aptitude vs Braton Incarnon are condition rules of `04`. | out of scope |
| EC-09-65 | Two innate primary elements on adversary weapons (HCET order) are `01`'s. | out of scope |
| EC-09-66 | PE `type = '---'` on Sniper Ammo Mutation (`/Lotus/Upgrades/Mods/Rifle/WeaponSnipersConvertAmmoMod`), its Intermediate copy (`…/Intermediate/WeaponSnipersConvertAmmoModIntermediate`) and Primed Sniper Ammo Mutation (`…/Expert/WeaponSnipersConvertAmmoModExpert`), `compatName` "Sniper": the slot type comes from `M:Mods.Type` (Sniper → PRIMARY). | E-9.4; `PE:Upgrades` |
| EC-09-67 | Mods with no PE record (53 outside Railjack and Plexus): the ten exalted stances, Butcher's Revelry, Clashing Forest, Primed Deadly Efficiency, Primed Steady Hands, Magnetic Rush, Magnetic Strafe, nine Flawed weapon mods; fields come from `M:Mods` (E-9.3a); the exalted stances' `M:Mods` values (`BaseDrain 0`, no PE `STANCE` record) contradict U38.5 and are replaced by X-STANCE. | E-9.3, E-9.3a; `W:Exalted_Weapon` L280 |
| EC-09-68 | Game vs wiki data: U38.5 and U38.6 made the Acolyte, Amalgam and combo-counter mods and Energizing Shot equippable on exalted weapons, but `M:Mods` still lists `POWER_WEAPON` in their incompatibility tags; X-EXALT lifts it (patch notes, precedence 2) and adds the trigger requirements (`NO_AIM`, `NO_SLIDE`). | E-9.5; `W:Exalted_Weapon` L156, L253-276, L299 |
| EC-09-69 | Kitgun chamber records of `M:Weapons` hold `Magazine = 0` (a placeholder); the pre-expanded `Modular/data/kitgun` combos have real magazines but empty `CompatibilityTags`. Tags and trigger come from the chamber record, the base magazine from `06` E-6.30; Synth Charge legality therefore depends on the loader (UNVERIFIED). | E-9.1, E-9.7; OQ-920 |
| EC-09-70 | Mecha Empowered (aura) is a Mecha set piece but does not count towards N_Mecha (in-game bug the engine follows). | E-9.40; `W:Mecha_Empowered` L42; `04` OQ-439 |
| EC-09-71 | Dark Split-Sword has two records with different `Family` values; one riven fits both forms (X-FAMILY). | E-9.51 |
| EC-09-72 | `Family` is missing on Nunchasa, Aksondol, Sagek Prime and Thermian RPG (and on Lizzie, Neutralizer, Glory, Noctua, Ironbride, which take no rivens); the riven family falls back to the weapon name. | E-9.51 |
| EC-09-73 | `M:Mods.BaseDrain` and `MaxRank` disagree with PE on 44 non-riven records (Point Strike, Vital Sense 2 vs 4; Split Chamber 4 vs 10; Infected Clip 4 vs 6; Cryo Rounds 2 vs 6, which `W:Mod` L423 confirms as 11 at maximum rank; Reach and Charged Chamber maximum rank 5 vs 3); PE wins. | E-9.12; `PE:Upgrades` |
| EC-09-74 | Riven Initial Combo: the coefficient table footnotes it positive-only (L1108), the base-value table does not (L868), and the melee range table lists negative ranges (L312); engine: negative is `illegal_unverified`. | E-9.55 T3; OQ-921 |
| EC-09-75 | Garuda Talons: not exalted (free stance, exilus), but an arcane slot was added in U38.5 and rivens are not generated for them. | E-9.30, E-9.50; `W:Exalted_Weapon` L95, L106-107, L241 |
| EC-09-76 | Slot polarity strings: `Aura` = ANY; `Universal` = none (Dual Heat Swords stance slot, sourced; Haalvu exilus slot, UNVERIFIED); missing or empty = none (25 primary and secondary exilus slots). | section 6.1; `W:Dual_Heat_Swords` L29; OQ-922 |
| EC-09-77 | Melee Crescendo cannot go on Whipclaw, Shattered Lash, Landslide Fists (no finishers, `NO_FINISHER`). | E-9.30a; `W:Exalted_Weapon` L178-180 |
| EC-09-78 | Swap Polarity: the warframe exilus slot is swappable (an Umbra polarity "can be swapped to" the exilus and aura slots); the aura no-swap rule (U15.1) is older than that note and is kept (UNVERIFIED); weapon exilus and stance slots are not swapped (no source). | E-9.20; `W:Umbra_Forma` L68; `W:Aura` L352; OQ-911 |
| EC-09-79 | Cannonade on a Genesis weapon without its Genesis (Torid, Vasto, Dual Toxocyst): judged without the Incarnon form, so legal (UNVERIFIED); with the Genesis, illegal. | E-9.6; OQ-923 |
| EC-09-80 | Arquebex and Ironbride have `Class = 'Exalted Weapon'` but no `POWER_WEAPON` tag; `exalted(W)` also tests `Class`, so they take no rivens and count as exalted for Catalyst. | E-9.1; `W:Exalted_Weapon` L68-70; `W:Riven_Mods` L1526 |

---

## 17. Inspiration differences

Register rows of overview section 5.2 owned by this document, restated with the engine behaviour, plus new rows in
this document's range `D-901` … (the overview's `D-n01` form for document 09).

| ID | Topic (owner) | Inspiration behaviour | Game behaviour (source) | Decision | Toggle |
|---|---|---|---|---|---|
| D-092 | Mod capacity, polarity and rank (09) | no drain, capacity, polarity, Forma or Catalyst; every mod at its table (maximum-rank) value (`I:opt` §2, §8; no "capacity"/"polarity" in `JS L15252-22038`) | drain per rank, polarity halving and penalty, capacity by rank and Catalyst, Forma (`W:Mod` L156-167, `W:Polarity`, `F:mods` §2-3) | E-9.11 to E-9.20; capacity mode `any_forma` by default; `ignore` reproduces the inspiration | User (T-35, T-36) + Compat (capacity mode `ignore`) |
| D-093 | Mod incompatibility and duplicates (09) | duplicate guard `isModAlreadyAdded`: any installed name **containing** the candidate's name blocks it (`JS L6998-7007`), which also blocks unrelated pairs; `isModCompatible` checks only the installed mods' `uncomp` lists (one direction, `JS L6987-6997`); `uncomp` data is asymmetric and names non-existent mods (`I:model` Q18-Q19); `showMod` never blocks a duplicate, so links can carry duplicates (`JS L7267-7272`, `I:feat` quirk 7); compatibility by hand-assigned category tags (`JS L7136-7230`) | same parent type ⇒ exclusive; wiki `Incompatible` lists (`W:Mod/Compatibility` L261-262, `F:mods` §5) | E-9.9 (union of both directions), E-9.10 D1-D2, E-9.4-E-9.8 from wiki data | — |
| D-901 | Riven legality (09) | up to 4 rows, each a distinct trait of the weapon category's list (a chosen trait is hidden from the other rows, `JS L14753-14766`, `y.includes(K)`) with at most 3 of Heat, Cold, Electricity, Toxin (`3 > J`); any value −9999 … 9999; no layout, sign, class, range or disposition rules; the riven pseudo-mod is offered on every weapon incl. exalted (`JS L7273-7341`, `L14685-14845`; `I:feat` §6) | 2-3 positive and 0-1 negative traits, sign rules, class availability, ranges from E-4.18 with D = 1.2, no rivens on exalted weapons (`W:Riven_Mods`) | validate E-9.50 to E-9.59, flag, evaluate the user's values unchanged | — (validation only; numbers unaffected) |
| D-902 | Riven trait catalogue (04, 09) | one catalogue object for every weapon (`JS L14696-14738`), from which melee deletes the gun-only keys (Punch Through, Shot Speed, Reload, Magazine, Ammo Maximum, Multishot, Ammo Efficiency) and guns delete Range, Heavy Attack Efficiency, Initial Combo and Combo Duration (`JS L14739-14750`). Present: Corpus, Grineer, Infested and the splices Scaldra and Techrot; the six combined-element splices (`addViral` …); Status Damage; Weak Point Damage (mis-mapped to the headshot bucket, D-011); Weak Point Critical Chance (stored under `SMITE`, D-010); Ammo Efficiency; Heavy Attack Efficiency (melee). The ranged splices (Weak Point ones included) and Zoom and Recoil are also offered on melee. Missing: Orokin (D-408), Reload While Holstered, Heavy Attack Wind Up, Melee Damage On Heavy Attack, Parry Angle, Slam Attack Damage, Finisher Damage, Critical Chance for Slide Attack, Additional Combo Count Chance, Chance to Gain Combo Count | per-class trait table and 18 splices (`W:Riven_Mods` L809-985, L1312-1398) | full per-class catalogue (section 12; coefficients `04` §8.3); link import maps every inspiration key, the `add*` splices included (section 2.4) | — |
| D-903 | Set-bonus counting (09) | `getCountSetMods` counts equipped panel slots with the same `set.name` (`JS L8514-8518`; `set` data only on the Sacrificial pair); Vigilante and Gladiator are per-piece stats on each mod; duplicates counted, no companion pieces, no active-weapon rule | distinct pieces on warframe + active weapon (+ companion and robotic weapon, deduplicated) since U38.5 (`W:Set_Mods` L217-224, L257-259, L406) | E-9.40 | — |
| D-904 | Archon shards (09, 04) | 18 fixed shard entries (Crimson CD/SC/CC, Topaz CC, Violet CD/Electricity/ability, Emerald stacks/Toxin), offered by weapon category tags; no Amber, Azure or Crimson Strength/Duration entries (`I:model` §4.4), so Azure shards cannot raise the Violet count | 5 slots, any colour, one of its bonuses each (`W:Archon_Shard` L181, L244-400) | E-9.31 | — |
| D-905 | Valence value (01, 09) | any v in [0.25, 0.60], clamped at the input (`JS L14641-14654`); the optimizer enumerates elements at the page value (`I:opt` §5.1) | 0.1 % steps; ≥ 58 % becomes 60 % (`W:Valence_Fusion` L21, L96-99) | E-9.70: 58.0-59.9 % flagged `illegal` (Coda `illegal_unverified`), evaluated as entered | — |
| D-906 | Cannonade gating (09) | `trigger: "Semi"` mods need the weapon's single `trigger` field to be `"Semi"` (`JS L7203-7208`); per-attack triggers, Incarnon forms and kitgun rules are not read | every trigger component of every mode must be Semi-Auto; primary kitguns excluded, secondary kitguns allowed (`W:Cannonade_Mods` L5; `W:Semi-*_Cannonade` notes) | E-9.6 | — |
| D-907 | Kitgun and zaw arcane slot (09) | one weapon-arcane slot (10); no Pax, Residual or Exodia arcanes in the data (`I:model` §1.2; arcane entries `JS L4499-4815` and `JS L5172-5254`) | separate kitgun (Pax/Residual) and zaw (Exodia) slot (`W:Arcane_Enhancement` L22) | E-9.30 | — |
| D-908 | Warframe layout (09) | 4 warframe-mod slots (13-16), no warframe exilus slot; the optimizer uses only slot 13 (`I:feat` §5.1, `I:opt` §2) | 8 general + exilus + aura (Jade 2) (`W:Mod` L143) | full layout; link slots 13-16 map to `fgen0`-`fgen3` | — |
| D-909 | Offering of auras, warframe mods, shards and arcanes (09) | filtered by weapon category tags (Steel Charge only on melee, `JS L7136-7230`, aura entries `JS L4816-4897`) | any aura or warframe mod can be equipped; effects are conditional (`W:Mod/Compatibility` L286-289) | offer all legal items; effect gating per `04` | — |
| D-910 | Helminth restriction (09) | buffs are free toggles; Roar, Eclipse and Xata's Whisper can all be enabled at once (`I:feat` §8) | self-cast combinations restricted (`W:Helminth` L251, L259-278) | E-9.32 only for buffs marked self-cast; T-14 … T-16 are always self-cast, T-13 is ally-cast by default (T-37) | — |
| D-911 | Synth Charge equip rule (06, 09) | equippable on any secondary (tag `secondary`, `JS L2171-2176`); effect applied only when modded shots per magazine `H` exceed 6 (`JS L12480-12481`, `L12519-12521`) | equip needs base magazine ≥ 6 (Kulstar bug); effect still works when the magazine is lowered later (`W:Synth_Charge` L27-29, L41) | E-9.7; effect rule `06` EC-06-48 | — |
| D-912 | Exalted-weapon mod gating (09) | only ten mods carry `uncomptag` `POWER_WEAPON` (`JS L790-L4954`, for example Laser Sight, Split Flights, Harkonar Scope; `I:model` §1.1), so Blood Rush, Weeping Wounds, Gladiator Rush and the other U38.5 mods are not excluded from exalted weapons, while Harkonar Scope is hidden on every weapon whose `compTags` hold `POWER_WEAPON` (Neutralizer too if its record carries the tag; the weapon data are not in the bundle); no aim, slide or finisher requirement is read | `M:Mods` `POWER_WEAPON` tags as amended by U38.5 and U38.6 (X-EXALT), with `NO_AIM` and `NO_SLIDE` requirements (`W:Exalted_Weapon` L156, L253-276, L299) | E-9.5 with X-EXALT | — |

The riven faction-trait list (`D-408`), the riven Weak Point traits (`D-010`, `D-011`) and the incarnon perk attack
filter (`I:model` Q22) are owned by `04`.

---

## 18. Open questions

| ID | Question | Current decision | Evidence that would settle it |
|---|---|---|---|
| OQ-901 | Non-matching drain: round half up (Mod page) or round up (Polarity page)? | `floor(1.25d + 0.5)` | Point Strike at rank 5 (drain 9) in a Naramon slot: arsenal shows 11 or 12 |
| OQ-902 | Aura/stance mismatch bonus: 80 % floor or 75 % rounded? | `floor(0.8b)` | a stance at rank 0 (bonus 2) in a mismatched slot: 1 vs 2 (rank 1, bonus 3, gives 2 under both); an aura with bonus 6 (Corrosive Projection rank 4): 4 vs 5 |
| OQ-903 | Drain of a non-Umbra mod in an Umbra slot; Umbra mod in an ANY slot | mismatch; mismatch | arsenal readout of Pressure Point in an Umbra slot and of Sacrificial Steel in an ANY slot |
| OQ-904 | One riven per weapon: no wiki statement | enforced | in-game attempt to equip a second riven of the same weapon; or a patch note |
| OQ-905 | Riven drain: base value range and growth per rank | user input d_u at rank r_u, `ξ(r) = d_u − r_u + r`; default `ξ(8) = 18`, polarity unknown (E-9.59) | drain readout of several rivens at ranks 0 and 8 |
| OQ-906 | Arcane slots of Dex Pixia (a Secondary record that acts as Razorwing's archgun) and of the Necramech exalted weapons Arquebex and Ironbride (U38.5 says "all Exalted weapons", `W:Arcane_Enhancement` L22 says Necramechs have none). Exalted primaries, secondaries, melee and pseudo-exalted weapons are settled by U38.5 (`W:Exalted_Weapon` L239, L312) | Dex Pixia secondary; Arquebex and Ironbride none | arsenal check of Dex Pixia, Arquebex and Ironbride |
| OQ-907 | Can Duplex weapons (Tigris, Zylok) equip Cannonade? Are per-attack trigger fields complete for alt-fires? | no; inherit the weapon trigger | in-game attempt; data audit of compound triggers |
| OQ-908 | Minimum capacity formula and Legendary-Rank capacity (three pages disagree) | ignored; `capacity_override` | capacity readout of a rank-30 and an unranked weapon on an MR 30 + LR n account, with and without Catalyst |
| OQ-909 | Valence hidden precision; does a new 58-59.9 % roll still snap to 60 % at drop time; does the U36 snap (named for Kuva and Tenet) apply to Coda weapons? | display tenths are exact; snap at all times; Coda as Kuva and Tenet (`illegal_unverified`) | fuse a weapon shown 52.7 % several times and record 57.9 % vs 60 %; collect fresh drops, Coda ones included |
| OQ-910 | Physical riven trait threshold: share of which attack, and the exceptions | share of the default attack, warning only | a list of weapons whose rivens roll a physical type below 25 % |
| OQ-911 | Can Swap Polarity move exilus, aura or stance polarities; can a Forma re-apply the same polarity (overlevel Forma count)? Evidence: `W:Umbra_Forma` L68 (an Umbra polarity "can be swapped to" a warframe's exilus or aura slot) contradicts the U15.1 aura note (`W:Aura` L352); U38.5.2 removed the swap of the exalted stance polarity "since the Polarity is designed to match the associated Stance" (`W:Exalted_Weapon` L208-209), which suggests ordinary stance polarities do swap | warframe exilus swaps; aura, weapon exilus and stance do not; not needed | arsenal check: swap into a warframe exilus and aura slot, a weapon exilus slot and a melee stance slot |
| OQ-912 | Do Kuva, Tenet and Coda variants share the base weapon's riven (as `Family` says)? | yes | equip a Hek riven on Kuva Hek |
| OQ-913 | Are multi-tag `CompatibilityTags` "any of"? | yes (data evidence) | equip Sniper Ammo Mutation on a sniper rifle (it has `SNIPER_AMMO`, no weapon has `LAUNCHER_AMMO`) |
| OQ-914 | Can Longbow Sharpshot, Shotgun Vendetta and Akimbo Slip Shot be equipped on other weapons of their slot? | yes, no effect | in-game attempt |
| OQ-915 | Toggles for capacity mode (`ignore`, `current`, `any_forma` default, `budget(N)`, allowed Forma types, `frame_complete` for warframe capacity), lower mod ranks in the optimizer (default off) and a caster field (`self`, `ally`; default `ally`) for T-13 … T-16 | adopted as overview T-35, T-36 and T-37 (2026-11-24) (T-37 narrowed to T-13 by OQ-926) | settled |
| OQ-916 | Compatibility nodes of exalted primaries without a wiki statement (Lizzie) | Primary, Rifle, Assault Rifle | arsenal mod list of Lizzie |
| OQ-917 | Does a missing `CompatibilityTags` field mean "no tags" (72 guns)? | yes | in-game mod list of Higasa (Rifle without tags): is Rifle Ammo Mutation (needs `ASSAULT_AMMO`) offered? |
| OQ-918 | Riven value display: round or truncate (tolerance size); is W computed in f32 before the product (`04` OQ-424)? | one display unit tolerance; per E-4.18 | riven readouts at the ends of a range (from trait locking and splicing) |
| OQ-919 | Exalted weapons: is the X-EXALT list complete? Galvanized Crosshairs (the Galvanized counterpart of Hydraulic Crosshairs, not named in U38.5) and any later mod; which mods need aim, slide, finisher or attack speed | Galvanized Crosshairs keeps `POWER_WEAPON`; requirements as in X-EXALT | arsenal mod list of Regulators, Balefire Charger, Exalted Blade and Landslide Fists |
| OQ-920 | Does Synth Charge legality on a secondary kitgun follow the assembled magazine (loader), or is it decided per chamber? | assembled `Mag0` (E-9.7) | equip Synth Charge on Catchmoon (Secondary) with the Zip loader (5 rounds) and with Arcroid (9) |
| OQ-921 | Can riven Initial Combo roll negative (range table L312) or not (footnote L1108)? | negative is `illegal_unverified` | a melee riven readout showing a negative Initial Combo, or a patch note |
| OQ-922 | Haalvu's exilus polarity is `Universal` in `M:Weapons`: unpolarized (as Dual Heat Swords' stance slot) or Any? | unpolarized | arsenal view of Haalvu's exilus slot |
| OQ-923 | Is Cannonade legality of a Genesis weapon judged with or without its Incarnon form when the Genesis is not installed (Torid, Vasto, Dual Toxocyst)? | without (legal until the Genesis is installed) | equip Semi-Rifle Cannonade on a Torid without its Genesis |
| OQ-924 | Riven drain default when a link carries none | `ξ(8) = 18` (conservative); see OQ-905 | drain readouts of a sample of rivens at rank 8 |
| OQ-925 | Proposals to other documents: `04` §1.5 to reference X-RANK (E-9.13) instead of keeping its own arcane rank table (adopted by `04` 2026-11-24); `04` §8.2 to note that Parry Angle shows no % sign; `04` to catalogue Zid-An Uskos (+2.50 non-combining Heat on secondaries) as an external buff | as stated in this document | owners' decisions |
| OQ-926 | (Closed 2026-11-24: adopted as overview T-37, which covers T-13 only.) Proposal to the manager: T-37 (caster of T-13 … T-16) should apply to T-13 (Roar) only; Eclipse, Xata's Whisper and Toxic Lash affect only the caster's weapons (`W:Eclipse` L9, `W:Xata's_Whisper` L8, `W:Toxic_Lash` L14), so T-14 … T-16 are always self-cast and E-9.32 always applies to them (T-16 already implies Saryn is the frame, D-037) | adopted: T-37 covers T-13 (default `ally`); T-14 … T-16 always self-cast | settled |

---

## 19. Test vectors

Each vector gives every input needed; data values were read from `M:Mods`, `M:Weapons`, `M:Modular`, `PE:Upgrades`
and `PE:Arcane` (cached 2026-11-23); drains are PE `baseDrain` + rank (E-9.12). Arithmetic was checked with
`python3` (capacity and Forma by brute force over all slot assignments; riven ranges with numpy float32 step by step).

**TV-09-01 Drain and polarity (E-9.12, E-9.14, E-9.15).**

| Input | Expected |
|---|---|
| Serration base 4, r = 10: ξ | 14; matching slot 7; unpolarized 14; mismatch 18 (alternative rule also 18) |
| Point Strike base 4, r = 5, Madurai mod in a Naramon slot | ξ = 9; mismatch 11 (alternative 12) |
| d = 1, 5, 13, 17 mismatch | 1, 6, 16, 21 (alternative 2, 7, 17, 22) |
| d = 0 any slot | 0 |
| Sacrificial Steel (UMB) d = 16 in an ANY slot | mismatch 20 |
| Serration d = 14 in an ANY slot | 7 |
| Blind Justice base −2, r = 3 (b = 5): matching, unpolarized, mismatch | 10, 5, 4 (alternative 4) |
| stance r = 0 (b = 2) mismatch | 1 (alternative 2) |
| Steel Charge base −4, r = 5 (b = 9) | matching 18, mismatch 7 |

**TV-09-02 Capacity and minimum Forma, no innate polarities (E-9.16 to E-9.19).** Braton Prime (`Polarities` nil,
`ExilusPolarity` Naramon, `MaxRank` 30), Catalyst, rank 30. Mods (ξ, polarity): Serration 14 M, Galvanized Chamber
16 M, Primed Bane of Grineer 14 M, Point Strike 9 M, Vital Sense 9 M, Hellfire 11 N, Infected Clip 11 N, Hunter
Munitions 9 M.

| Case | Expected |
|---|---|
| (a) exilus empty | C_cap = 60; unpolarized U = 93; N_forma* = 6, U = 57; best U with 5 Forma 61; `cap.exceeded` false in `any_forma` and `budget(6)`, true in `budget(5)` and `current` |
| (b) Vigilante Supplies (N, ξ 9, exilus) in the Naramon exilus slot | exilus cost 5 (matching); N_forma* = 7, U = 58 |
| (c) Gun Glide (M, ξ 9, exilus, Type Assault Rifle) in the Naramon exilus slot | kept: 11 (mismatch), which needs 8 general Forma (U = 60); with an exilus Forma: 5 and 7 general Forma; N_forma* = 8, U = 58 (exilus Forma chosen by the smaller U) |

**TV-09-03 Capacity with an innate polarity and an infeasible build.** (a) Lex Prime (`Polarities` [Madurai],
`MaxRank` 30), Catalyst, rank 30. Mods: Hornet Strike 14 M, Galvanized Diffusion 14 M, Pistol Gambit 9 M, Primed
Target Cracker 14 M, Lethal Torrent 11 M, Primed Heated Charge 16 N, Pistol Elementalist 9 V, Synth Charge 9 M.
Expected: unpolarized sum 96; N_forma* = 5, U = 58 (the innate Madurai slot holds a 14-drain Madurai mod; Forma on
Primed Heated Charge, two of the remaining 14s, Lethal Torrent and one 9). Same result under the alternative mismatch
rule. (b) A non-melee weapon (no stance slot) with C_cap = 60 and eight Madurai mods of drain 16: all-matched U = 64 >
60, no layout exists, `cap.exceeded` true in every mode except `ignore`. (c) The same eight mods on a melee weapon with
C_cap = 60 and a maxed stance in a matching stance slot (β = 10): N_forma* = 8, U = 54, feasible.

**TV-09-04 Stance capacity and exalted capacity (E-9.3, E-9.3a, E-9.15, E-9.16).** (a) Nikana Prime (`StancePolarity`
Madurai), Catalyst, rank 30, Blind Justice (M) rank 3 in the stance slot: β = 10, so mods may use U_gen ≤ 70. Same
stance in an unpolarized stance slot: β = 5 (65); in a Vazarin slot: β = 4 (64). (b) Exalted Blade: Catalyst forced,
rank 30, C_cap = 60. Its stance "Exalted Blade" has no PE record and `M:Mods` gives `BaseDrain 0`, `MaxRank 3`
(`InternalName …/MeleeTrees/DoomSwordMeleeTree`); E-9.3a alone would give kind stance (`MeleeTree`) but ξ(3) = +3. With
X-STANCE: kind stance (passes `slot.kind` in the stance slot), baseDrain −2, rank 3, b = 5, Zenurik in the fixed
Zenurik slot → β = 10, so mods may use U_gen ≤ 70. Blind Justice in Exalted Blade's stance slot → illegal
(`slot.kind`, stance_fixed).

**TV-09-05 Overlevel (E-9.16).** Kuva Hek (`MaxRank` 40): rank 40, Catalyst → C_cap = 80; rank 36 → 72; rank 31 → 62;
rank 40 without Catalyst → 40; Forma lower bound for rank 40: 5; for rank 36: 3; for rank 31: ceil(1/2) = 1. Braton
Prime rank 40 → `cap.rank` illegal (MaxRank 30).

**TV-09-06 Rank scaling (E-9.13, E-9.12; X-RANK).**

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
| Melee Duplicate (affine), r = 2 | 0.55 |
| Melee Doughty (affine), r = 0 | 0.5 per 10 % Puncture status chance |
| Critical Precision (affine), r = 0 | 0.025 per headshot |
| Double Tap, R = 3, r = 1 | 0.10 per stack, stack cap 40, duration 1 s |
| Primary Bulwark, r = 0 | cap 2.50 (rate 0.01 per armor point) |
| Melee Careen ×2.50 at R = 5, r = 0 | ×1.25 (b = 1.50·1/6 = 0.25) |
| Secondary Fortifier (affine in b), r = 0 | ×3 |
| Arcane Crepuscular, R = 5, r = 3 / r = 2 | final crit damage +2.0 and "+1 Arcane Revive" present / +1.5 and absent |
| drains: Galvanized Chamber r 10; Blind Justice r 0; Steel Charge r 5; Synth Charge r 3 | 16; −2; −9; 9 |

**TV-09-07 Compatibility (E-9.4 to E-9.8, X-EXALT).**

| Mod → weapon | Expected | Rule |
|---|---|---|
| Serration (Type Rifle) → Braton Prime (Primary/Rifle) | legal | |
| Primed Chamber (Type Sniper) → Braton Prime | illegal | `compat.type` |
| Primed Chamber → Rubico Prime (Sniper Rifle); Primed Chamber + Charged Chamber together | legal; legal | |
| Tainted Mag (Assault Rifle) → Paris (Bow) | illegal | `compat.type` |
| Split Flights (Bow) → Lenz (tags incl. CRPBOW) / → Paris (tags PROJECTILE) | illegal / legal | `compat.tags` |
| Sinister Reach (Primary, needs BEAM) → Amprex (BEAM) / → Braton Prime / → Gaze (Primary) kitgun (chamber tag BEAM) | legal / illegal / legal | `compat.tags` |
| Ruinous Extension (Pistol, needs BEAM) → Gaze (Secondary) / → Catchmoon (Secondary) | legal / illegal | `compat.tags` |
| Synth Charge → Lex Prime (Magazine 8) / Furis (35) / Kulstar (3) / Knell (1, SINGLESHOT) / Epitaph (SINGLESHOT) / Balefire Charger (POWER_WEAPON) | legal / legal / legal (bug list) / illegal / illegal / illegal | `compat.magazine`, `compat.tags` |
| Synth Charge → Catchmoon (Secondary) with loader Zip (tier Lowest, Mag0 5) / with Arcroid (Med, Mag0 9) | illegal / legal (UNVERIFIED, OQ-920) | `compat.magazine` |
| Semi-Rifle Cannonade → Latron ("Semi-Auto", Incarnon form "Semi-Auto") / Argonak ("Auto / Semi") / Nagantaka ("Semi / Mag Burst") | legal / illegal / illegal | `compat.semi` |
| Semi-Rifle Cannonade → Torid with `genesis_installed` (Incarnon Form "Held") / without | illegal / legal (UNVERIFIED, OQ-923) | `compat.semi` |
| Semi-Pistol Cannonade → Lex Prime / Laetum ("Semi / Auto") / Catchmoon (Secondary) | legal / illegal / legal | `compat.semi` |
| Semi-Shotgun Cannonade → Strun (Incarnon form inherits "Semi-Auto") / Catchmoon (Primary, MODULAR_GUN) | legal / illegal | `compat.semi`, `compat.tags` |
| Scattered Justice (Type Hek) → Hek / Kuva Hek / Vaykor Hek (SYNDICATEMODBLOCKED) | legal / legal / illegal | `compat.tags` |
| Blind Justice (stance, NIKANAS_STANCE) → Nikana Prime stance slot / a Nikana Prime general slot / Dragon Nikana stance slot | legal / illegal / legal | `slot.kind` |
| Bite (beast claw mod, Type Claws) → Garuda Talons (melee Claws) | illegal | `compat.type` |
| Sniper Ammo Mutation (PE type `---`, `M:Mods.Type` Sniper → PRIMARY; needs SNIPER_AMMO or LAUNCHER_AMMO; exilus) → Rubico Prime exilus slot | legal (regression case of EC-09-66) | `compat.type`, `compat.tags` (any of) |
| Blood Rush, Gladiator Rush, Weeping Wounds (`POWER_WEAPON` in `M:Mods`) → Exalted Blade | legal (X-EXALT lift) | `compat.tags` |
| Maiming Strike → Exalted Blade / → Shadow Clones (`NO_SLIDE`) | legal / illegal | `compat.tags` (exalt_req) |
| Hydraulic Crosshairs → Regulators (`NO_AIM`) / → Balefire Charger | illegal / legal | `compat.tags` (exalt_req) |
| Energizing Shot (Type "Pistol (No AoE)", incompatible AOE) → Regulators / → Balefire Charger (AOE) / → Artemis Bow (a primary) | legal / illegal / illegal | `compat.tags`, `compat.type` |
| Harkonar Scope (Type Sniper) → Neutralizer | legal (lift for Neutralizer) | |
| Galvanized Crosshairs → Regulators | illegal (not lifted; UNVERIFIED, OQ-919) | `compat.tags` |

**TV-09-08 Exclusion and duplicates (E-9.10, E-9.9).** On one weapon: Serration + Amalgam Serration → illegal
(`excl.pair`); Split Chamber + Galvanized Chamber → illegal; Critical Delay + Point Strike → illegal; Heavy Caliber +
Serration → legal; Vigilante Armaments + Split Chamber → legal; Electrified Barrel + Primed Electrified Barrel (only
the Primed mod lists the other) → illegal; Serration twice → illegal (`dup.mod`); Vigilante Supplies in `exilus` and
again in `gen0` → illegal (`dup.mod`); Sniper Ammo Mutation in `exilus` and Primed Sniper Ammo Mutation in `gen3` →
illegal (`excl.pair`); two rivens → illegal (`dup.riven`). On a warframe: Arcane Avenger twice → illegal
(`dup.arcane`); five identical Tauforged Violet shards with the primary-Electricity bonus → legal.

**TV-09-09 Set counting (E-9.40, E-9.41).** Warframe mods: Vigilante Pursuit, Vigilante Vigor. Primary: Vigilante
Armaments, Offense, Fervor, Supplies. Declared robotic-weapon pieces: Vigilante Armaments (companion_kind sentinel).

| Evaluated attack on | Expected N_Vigilante |
|---|---|
| the primary | 6 |
| the primary, robotic piece removed | 6 |
| the primary, warframe mods removed | 4 |
| the melee weapon (primary holstered) | 2 |
| the primary, Vigilante Armaments entered twice through a link (illegal duplicate) | 6 (identities counted once) |

Gladiator: Might, Vice, Rush on the melee weapon and Aegis, Finesse, Resolve on the warframe → 6 on a melee attack, 3
on a primary attack. Gladiator Rush and Gladiator Might on Exalted Blade, Gladiator Vice on the regular melee weapon,
Aegis and Finesse on the warframe: an Exalted Blade attack → 4; a regular melee attack → 3. Sacrificial Steel and
Pressure on the melee weapon → 2. Mecha: Mecha Empowered in `aura0` and Mecha Pulse → N_Mecha = 1; plus declared
Mecha Recharge and Mecha Overdrive with companion_kind kubrow → 3; the same declared pieces with companion_kind
sentinel → illegal (`set.companion`, Kubrow mods on a sentinel).

**TV-09-10 Riven value ranges (E-9.53 to E-9.56; E-4.18 in binary32).**

| Riven | Trait | Layout, rank | Range of v (100·v %) | Input → result |
|---|---|---|---|---|
| Braton (Rifle) | Critical Chance b 0.016666 | 2+1, 8 | 200.4670 … 245.0152 | +220.0 → legal (u 0.4385) |
| Braton (Rifle) | Multishot b 0.01 | 2+1, 8 | 120.2850 … 147.0150 | +130.0 legal; +147.1 legal (≤ v_hi + tol = 147.1150); +147.2 → illegal; +150.0 → illegal (u 1.1117) |
| Braton (Rifle) | Damage b 0.0183333 (negative) | 2+1, 8 | −107.8108 … −88.2088 | −95.0 → legal |
| Braton (Rifle) | Critical Chance | 2+1, 0 | 22.2741 … 27.2239 | — |
| Braton (Rifle) | Faction b 0.005 | 2+0, 8 | ×1.48114 … ×1.58806 | ×1.53 → legal |
| Braton (Rifle) | Faction (negative) | 2+1, 8 | ×0.70597 … ×0.75943 | — |
| Braton (Rifle) | Weak Point Damage splice b 0.025 | 2+1, 8 | 300.7125 … 367.5375 | — |
| Braton (Rifle) | Weapon Recoil b −0.01 (positive role) | 2+1, 8 | −147.015 … −120.285 | −130.0 → sgn = +1, legal |
| Braton (Rifle) | Ammo Efficiency splice b −0.005 (positive role) | 2+1, 8 | −73.5075 … −60.1425 (card +60.1 … +73.5 %) | card +65.0 % → v = −0.65, sgn = +1, legal |
| a melee riven | Chance to Gain Combo Count b −0.01165 (negative role) | 2+1, 8 | 56.0528 … 68.5090 (card −56.1 … −68.5 %) | card −60.0 % → v = +0.60, sgn = −1, legal; card +60.0 % → sgn = +1, illegal (T3) |
| Braton (Rifle), spliced 3+1 | Viral splice b 0.01; Critical Chance; Multishot; Zoom b 0.006666 (negative) | 3+1, 8 | 91.125 … 111.375; 151.8689 … 185.6176; 91.125 … 111.375; −59.3941 … −48.5951 | +100.0, +170.0, +100.0, −50.0 → legal (4 traits) |
| Catchmoon kitgun (Pistol column) | Damage b 0.0244 | 3+0, 8 | 177.876 … 217.404 | +195.0 → legal (u 0.4332) |
| Catchmoon kitgun | Damage | 3+0, 5 | 118.584 … 144.936 | — |
| a melee riven | Critical Chance b 0.02 | 2+0, 8 | 192.456 … 235.224 | — |
| Deconstructor (Companion riven, Class Glaive → Melee column) | Critical Chance b 0.02 | 2+0, 8 | 192.456 … 235.224 | — |
| a melee riven | Melee Damage b 0.0183 | 2+1, 8 | 220.1215 … 269.0374 | — |
| a melee riven | Initial Combo b 0.27224 | 3+0, 8 | 19.8463 … 24.2566 (absolute) | — |
| a melee riven | Initial Combo (negative role) | 2+1, 8 | −16.0093 … −13.0986 (absolute) | −14.0 → `illegal_unverified` (T3, OQ-921) |

The Rifle rows reproduce the wiki overview table (`W:Riven_Mods` L56-80: Critical Chance 200.5 — 245.0, Damage
−88.2 — −107.8, Faction x1.48 — x1.59 and x0.71 — x0.76, Multishot 3+1 91.1 — 111.4, Critical Chance 3+1 151.9 —
185.6); the melee rows reproduce L290-320 (Chance to Gain Combo Count −56.1 — −68.5, Melee Damage 220.1 — 269.0,
Initial Combo −13.1 — −16.0).

**TV-09-11 Riven structure (E-9.50 to E-9.55).**

| Riven on weapon | Expected | Rule |
|---|---|---|
| Rifle riven, family Braton, on Braton Prime; CC+, MS+, Damage− | legal | |
| same riven on Mk1-Braton / on Braton Vandal | legal / legal | |
| Rifle riven family Cernos on Mutalist Cernos | illegal | `riven.binding` |
| Shotgun riven on Braton | illegal | `riven.binding` |
| Kitgun riven family Catchmoon on Catchmoon (Primary) and on Catchmoon (Secondary) | legal, Pistol column | |
| Melee riven family Dark Split-Sword on Dark Split-Sword (Dual Swords) and on Dark Split-Sword (Heavy Blade) | legal / legal (X-FAMILY) | |
| Rifle riven family Nunchasa on Nunchasa (`Family` missing) | legal (fallback to the name) | |
| any riven on Exalted Blade / on Garuda Talons / on Arquebex (Class Exalted Weapon, no tags) | illegal / illegal / illegal | `riven.binding` (class none) |
| Companion riven on Deconstructor | legal, Melee column | |
| Rifle riven with 4 positive traits | illegal (n_pos = 4) | `riven.layout` |
| Rifle riven with 2 negative traits | illegal (n_neg = 2) | `riven.layout` |
| Rifle riven: Viral (splice) +, Critical Chance +, Multishot +, Zoom − | legal (3+1 spliced, 4 traits) | |
| Rifle riven with Heat −40 % | illegal | `riven.trait` T3 |
| Melee riven with Melee Damage −100 % | illegal | `riven.trait` T3 |
| Rifle riven with Damage −95 % | legal (UNVERIFIED, T4) | |
| Melee riven with Multishot | illegal | `riven.trait` T2 |
| Rifle riven with Viral and Corrosive (two splices) | illegal | `riven.trait` T5 |
| Rifle riven with Parry Angle | illegal | `riven.trait` T2 (no Rifle coefficient; T5 also fails) |
| Rifle riven: Viral (splice) +, Cold +, Toxin + | legal | |
| Rifle riven with Impact + on Braton Prime (IMP 1.75 of 35 = 5 %) | legal with warning | `riven.physical` |

**TV-09-12 Valence (E-9.70, E-9.71).** Legal v10: 250, 579, 600; illegal: 249, 580, 595, 601. Fusion: (520, 400) →
572; (528, 250) → 600; (527, 527) → 579; (480, 300) → 528; (437, 437) → 480. Repeated self-fusion from a start value
until 600: 528 → 1 step, 480 → 2, 437 → 3, 398 → 4, 362 → 5, 330 → 6, 300 → 7, 273 → 8, 250 → 9; 436 → 4; 272 → 9.
Kuva Ghoulsaw (`Traits` contains Kuva Lich, `IsLichWeapon` absent) accepts a valence; Braton Prime with a valence →
illegal (`valence.weapon`); v10 = 580 → `valence.value`. v10 = 585 on Kuva Hek → `illegal`; on Coda Hema (`Traits`
Technocyte Coda) → `illegal_unverified`; 600 on Coda Hema → legal.

**TV-09-13 Archon shards (E-9.31).** (a) Shards: Violet (Tau, Electricity), Violet (Tau, Electricity), Crimson
(normal, Ability Strength), Azure (Tau, Health), Amber (normal, casting speed): legal; N_cav = 4. (b) Six shards →
illegal (`shard.count`). (c) Azure shard with "primary status chance" → illegal (`shard.bonus`). (d) Five Emerald
shards with the Corrosion max-stacks bonus → legal (repeats allowed).

**TV-09-14 Helminth (E-9.32).** (a) Self frame Rhino, subsumed Eclipse (self-cast), native Roar self-cast → illegal
(`helminth.host`). (b) Self frame Saryn, subsumed Roar and native Toxic Lash, both self-cast → legal (Saryn not a
host). (c) Self frame Chroma, subsumed Roar, ally-cast Vex Armor from another Chroma → legal. (d) Two subsumed
abilities → illegal (`helminth.count`). (e) Self frame Mesa, subsumed Xata's Whisper replacing Shatter Shield, native
Shooting Gallery → legal. (f) Frame identity unset, subsumed Eclipse and a self-cast Roar → legal (only H1 and H5 are
checked). (g) Self frame Rhino, subsumed Eclipse, toggle T-13 (Roar) on with T-37 at its default `ally` → legal (the
toggle counts as ally-cast); with T-37 = `self` for T-13 → illegal (`helminth.host`, as (a)).

**TV-09-15 Incarnon (E-9.60; import 2.4).** Furis with `genesis_installed`: `inc2` = Haven Foray (X = 28), `inc3` =
Extended Volley, `inc4` = Elemental Balance (X = 0.28) → legal; T = 4, so `inc5` set → illegal (`incarnon.tier`).
Mk1-Furis with the same choices → legal with X = 34 and 0.39. Two perks for `inc2` → illegal (`incarnon.tier`). Any
perk on Afuris (Akimbo) → illegal (`incarnon.line`). Furis without `genesis_installed` → the "Incarnon Form" attack is
not evaluable and perks are illegal (`incarnon.form`). Laetum (native): T = 5, `inc5` allowed. Import: a Furis link
with slot 19 = {mId 2, pId 1} and no slot 18 → `genesis_installed` true, `inc2` = the perk, `inc3` … `inc5` none; a
Furis link with slots 18-22 empty → `genesis_installed` true (default), every `inc_choice` none.

**TV-09-16 Arcane slots (E-9.30, E-9.30a).** Archgun (Imperator): `arcane0` Primary Merciless, `arcane1` Secondary
Shiver → legal; `arcane0` Secondary Shiver → illegal (`arcane.family`). Braton Prime `arcane1` set → illegal
(`slot.count`). Catchmoon (Secondary) kitgun: `arcane0` Secondary Merciless, `arcane1` Pax Charge → legal; `arcane1`
Exodia Contagion → illegal (`arcane.family`). Warframe: Arcane Avenger + Arcane Fury → legal; robotic weapon with any
arcane → illegal (`slot.count`). Exalted Blade `arcane0` Melee Crescendo → legal; Whipclaw `arcane0` Melee Crescendo →
illegal (`arcane.gate`); Garuda Talons `arcane0` Melee Duplicate → legal; Regulators `arcane0` Secondary Merciless →
legal; Arquebex with any arcane → illegal (`slot.count`, UNVERIFIED, OQ-906). Primary Merciless rank 6 → illegal
(`rank.range`, R = 5); Pax Charge rank 4 → illegal (R = 3).

**TV-09-17 Umbra polarities, current mode and warframe capacity (E-9.14 to E-9.20).**

| Case | Expected |
|---|---|
| (a) Skiajati (`Polarities` [Umbra, Umbra], `StancePolarity` Madurai), Catalyst, rank 30, Blind Justice (M, b 5) in the stance slot; Sacrificial Steel 16 UMB, Sacrificial Pressure 16 UMB, Primed Reach 14 M, Blood Rush 14 M, Condition Overload 15 M, Weeping Wounds 9 M, Gladiator Might 9 M, Primed Fury 14 M | the Umbra mods in the innate UMB slots cost 8 each; β = 10; U with 0 Forma 81; N_forma* = 3, U = 60 (Forma on Condition Overload and two 14s); `current` → `cap.exceeded` true |
| (b) Vinquibus (Melee) (`Polarities` [Madurai, Aura]): Sacrificial Steel (ξ 16) | in the ANY slot 20 and in the Madurai slot 20 (mismatches, UNVERIFIED, OQ-903), unpolarized 16, with one Umbra Forma (allowed through the melee form) 8 |
| (c) Vinquibus (Primary), the shared layout after that Umbra Forma ([Madurai, ANY, UMB]): Serration (ξ 14) | in the UMB slot 18 (mismatch), in the ANY or Madurai slot 7; the primary has no Umbra mods, so its E-9.19 never chooses an Umbra Forma |
| (d) Rubico Prime (`Polarities` [Madurai, Madurai], `ExilusPolarity` Naramon), Catalyst, rank 30: Serration 14 M, Split Chamber 15 M, Point Strike 9 M, Vital Sense 9 M, Hellfire 11 N, Primed Chamber 7 M; Sniper Ammo Mutation (N, ξ 7) in the exilus | Split Chamber and Serration in the innate Madurai slots (8 + 7), others unpolarized (36), exilus 4: U = 55 ≤ 60, `current` feasible, N_forma* = 0 |
| (e) Jade (`Polarities` [Naramon], `AuraPolarity` [Aura, Vazarin]), Reactor, rank 30, `frame_complete` true: Streamline 9 N, Continuity 7 M, Flow 9 N, Umbral Vitality 16 UMB, Umbral Intensify 16 UMB, Mecha Pulse 9 M, Stretch 9 N, Redirection 12 V; `aura0` (ANY) Corrosive Projection (N, b 7), `aura1` (Vazarin) Steel Charge (M, b 9) | β = 14 + 7 = 21; general 83 (one 9-drain Naramon mod in the Naramon slot: 5); U = 62 > 60 at 0 Forma (`current` → `cap.exceeded` true); N_forma* = 1, U = 51 (a Forma making `aura1` Madurai or ANY: β = 32); the best Umbra Forma alone gives U = 54 |
| (f) as (e) with the auras swapped (`aura0` Steel Charge, `aura1` Corrosive Projection) | β = 18 + 5 = 23; U = 60, `current` feasible, N_forma* = 0 |
| (g) as (e) with `frame_complete` false | no `cap.exceeded` check on the warframe |

**TV-09-18 Riven drain defaults (E-9.59, E-9.14).**

| Case | Expected |
|---|---|
| Rifle riven entered at rank 8 with drain 16, polarity Madurai, evaluated at rank 5 | ξ0 = 8, ξ(5) = 13; in a Madurai slot ceil(13/2) = 7; in a Naramon slot floor(16.25 + 0.5) = 16 |
| Riven imported from an inspiration link (no drain, no polarity), rank 8 | ξ(8) = 18, polarity unknown; unpolarized slot 18; Madurai slot 23; ANY slot 23 (unknown matches nothing); a Forma on its slot gives none (18), never 9; the capacity report names the defaults |
| Braton Prime of TV-09-02 (a) with Hunter Munitions replaced by that imported riven | unpolarized 93 − 9 + 18 = 102; `any_forma`: the riven's slot cannot be halved, so the six Forma of TV-09-02 (a) on the other mods give U = 57 − 9 + 18 = 66 > 60 and a seventh Forma (Point Strike or Vital Sense, saving 4) gives 62; N_forma* does not exist → `cap.exceeded` true in every mode but `ignore` |

---

## Review notes

Notes on the second review (2026-11-24): issues rejected in whole or in part, and changes beyond the issues.

- Symbols: the rank symbol r is kept (not renamed to r_rk) because `04` §1.5 and E-4.18 already use r for mod, arcane
  and riven ranks and the overview's ratio is always written r_t; every other colliding symbol was renamed (section
  1.4).
- TV request "Sinister Reach → Gaze (Secondary) legal" rejected as stated: Sinister Reach has `Type` Primary; the
  vectors use Sinister Reach → Gaze (Primary) and Ruinous Extension (Pistol, `BEAM`) → Gaze (Secondary).
- TV request "Energizing Shot → Artemis Bow legal" rejected: Energizing Shot is a pistol mod ("Pistol (No AoE)") and
  Artemis Bow a primary, so `compat.type` forbids it; the legal case uses Regulators.
- Example 1 savings: the order fix was applied, but the values are not "7, 8, 7, 3, 3, 5, 4, 4": rechecking the vector
  showed that the drafts used stale `M:Mods` drains (Point Strike, Vital Sense 7; Infected Clip 9) against section
  1.3's choice of PE; with PE drains the list is 7, 8, 7, 4, 4, 5, 5, 4 and the result is N_forma* = 6, U = 57
  (EC-09-73; TV-09-01, TV-09-02, OQ-901 and EC-09-01 changed for the same reason).
- Swap Polarity evidence: applied, but `W:Aura` L352 is the U15.1 note and older than the `W:Umbra_Forma` L68 note,
  not newer as the issue says; E-9.20 now records the conflict.
- Changes the issues ask of `04` (reference X-RANK instead of keeping an arcane rank table, Parry Angle in the
  conversion table, a Zid-An Uskos entry) cannot be made here; they are proposed in OQ-925.
- EC-09-59: the count of 62 `POWER_WEAPON` mods is still what `M:Mods` holds; what changed is that 16 of them are lifted
  by X-EXALT, which the row now says.

Follow-up round (2026-11-24), applying the overview's toggle assignments and checking every overview reference:

- Placeholder toggles replaced: `T-cap-mode` → T-35 (capacity mode, with `frame_complete`), `T-low-ranks` → T-36, the
  proposed caster field → T-37 (sections 1.3, 1.4, 1.5, 5.1, 6.5, 10, 17 D-092, OQ-915, TV-09-14 (g), which gains the
  `self` case). D-092's Toggle column now matches the overview ("User (T-35, T-36) + Compat").
- Every other overview reference was checked against the current overview: T-02, T-13 … T-16, T-24, T-26; D-010,
  D-011, D-092, D-093 (and `04`'s D-408); NP-2 (riven values in f32, owner `04` E-4.18); ST-1 … ST-7; the `04`
  (E-4.12, E-4.18, E-4.21), `06` (E-6.30) and `07` (E-7.42) equations. No other change was needed.
- OQ-925: `04` §1.5 now points to X-RANK (adopted); the Parry Angle and Zid-An Uskos proposals stay open.
- New OQ-926: T-37's `ally` value has no game counterpart for Eclipse, Xata's Whisper and Toxic Lash (self-only
  buffs); this document follows the overview's T-37 and proposes the narrower toggle (section 10).

Follow-up round (2026-11-24), T-37 narrowed by the overview:

- Overview T-37 now covers T-13 (Roar) only and T-14 … T-16 are always self-cast (adopted from OQ-926; `W:Roar` L14,
  `W:Eclipse` L9, `W:Xata's_Whisper` L8, `W:Toxic_Lash` L14 rechecked in the wiki cache). Sections 1.3 (caster and
  toggle rows) and 10, D-910, OQ-915 and OQ-926 (closed) follow; `04` §4.4 already matches. TV-09-14 (g) concerns
  T-13 only and is unchanged. The issue's sentence "T-14 and T-15 together are always illegal" was narrowed to "when
  the frame identity is set", since section 10 checks only H1 and H5 without it (TV-09-14 (f)).
