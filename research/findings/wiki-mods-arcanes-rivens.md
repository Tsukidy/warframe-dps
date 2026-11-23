Status: draft

# Catalog rules for mods, arcanes, rivens, incarnons and adversary weapons (game-truth side)

Scope: the rules a build calculator and optimizer must respect when it *chooses* upgrades: slot counts, capacity,
drain and polarity, mutual exclusion, conditional/stacking effects, set bonuses, riven stat generation, incarnon
evolution choices, Kuva/Tenet/Coda valence bonus, weapon arcanes, and which machine-readable source carries which
of these. How each bonus enters the per-hit damage formula (additive vs multiplicative buckets, CO-type behaviour) is
covered in `wiki-additive-vs-multiplicative-edge-cases.md` and `wiki-core-damage-formula.md`; this file only
repeats bucket facts where a catalog rule depends on them.

Research date 2026-11-23. Game state: Hotfix 44.1.4.1 (2026-10-27) is the newest patch the wiki lists
(`Update_44:_Iceblade_of_Narin.wiki`). All wiki pages were read as raw wikitext and are cached in
`research/external/cache/wiki/` (page name, `/` → `__`). Wiki Lua data modules were read from the JSON conversions
in `research/external/cache/poc/out/` (`Mods_data.json`, `Arcane_data.json`, `Weapons_data_*.json`), produced by
the data-source evaluation (`data-source-wiki-modules.md`). DE Public Export (PE) files are in
`research/external/cache/public-export/manifests/`. "U" = game update number as the wiki tags it with `{{ver|…}}`.

---

## 1. Slot layout per equipment class

Source: `Mod` page § Installation (cached `Mod.wiki` lines 129-160), `Arcane Enhancement` § Application,
`Archgun` intro, `Exilus Weapon Adapter`, `Mod/Compatibility` § Slot Compatibility.

| Equipment | General mod slots | Special slots | Arcane slots | Notes |
|---|---|---|---|---|
| Primary weapon (rifle, shotgun, sniper, bow, launcher, speargun, crossbow; primary kitgun) | 8 | 1 Exilus (unlocked by Exilus Weapon Adapter) | 1 Primary arcane (unlocked by Primary Arcane Adapter). Primary kitgun: **plus** 1 Pax/Residual (kitgun) slot | Kitgun arcanes "occupy a unique slot separate from Primary/Secondary/Melee Arcanes" (`Arcane Enhancement` § Application, § Primary Weapon Arcanes). |
| Secondary weapon (pistol, thrown, tome, dual pistols; secondary kitgun) | 8 | 1 Exilus | 1 Secondary arcane. Secondary kitgun: plus 1 Pax/Residual slot | same |
| Melee weapon (incl. zaw) | 8 | 1 Exilus + 1 Stance | 1 Melee arcane (Melee Arcane Adapter, needs *Whispers in the Walls*). Zaw: plus 1 Exodia slot | Melee exilus slot exists since U35 / Whispers in the Walls (`Exilus Weapon Adapter`: "To fuse an Exilus Weapon Adapter onto a melee weapon, the player must first complete the quest Whispers in the Walls"). The `Exilus Mods` page still says "Exilus slot is present on all Warframes and primary and secondary weapons" — **outdated** on that point. |
| Archgun | 8 | none | **2**: top slot takes a Primary arcane, bottom slot a Secondary arcane (Archgun Arcane Adapter, U40) | Archguns "are not considered either primary or secondary weapons" so weapon-class arcane effects such as Arcane Precision do not apply, but universal ones (Arcane Avenger, Arcane Hot Shot) do (`Archgun.wiki` line 16; U40 notes lines 121-125). |
| Archmelee | 8 | none | 0 | `Arcane Enhancement` § Application: "Archwings, Archmelees, and Necramechs do not have associated Arcanes and slots." |
| Robotic (sentinel/MOA/hound) weapon | 8 | none | 0 ("Sentinel weapons cannot equip Primary/Secondary/Melee Arcanes") | Uses "Companion Weapon Riven" (robotic class); set mods count toward set bonus (§8). |
| Exalted weapon | same as a normal weapon of that type | Exalted melee: stance slot permanently occupied (except Garuda Talons); pseudo-exalted melee have no exilus | UNVERIFIED (see Open questions). `Primary Merciless` notes imply primary exalted weapons can hold Primary Merciless ("Stacks … not shared between different weapons that have Primary Merciless equipped, such as primary Exalted Weapons and Archguns"). | Rivens never generate for exalted weapons (`Riven Mods` § Notes). |
| Warframe (for weapon-affecting mods/arcanes) | 8 | 1 Exilus, 1 Aura (Jade 2 auras) | 2 Warframe arcanes | Needed because set mods, auras and warframe arcanes (Arcane Avenger, Arcane Rage …) buff weapons. |
| Companion | 10 | none | 0 | Companion mods count for Hunter/Mecha/Tek/Strain/Synth sets. |
| Beast claws | 8 | 1 Posture | — | |

Rules that apply to all slots (`Mod` § Standard Mods and Variants; `Mod/Compatibility` § Slot Compatibility):

- No duplicate mod on one item, and no two variants of the same standard mod (§6).
- Exilus mods may go in any slot; only Exilus mods may go in the Exilus slot. Aura/Stance mods only in their slot.
- Arcanes: "duplicate Arcanes cannot be equipped on the same item" (`Arcane Enhancement` § Application). This
  matters only for the two warframe arcane slots and the archgun's two slots.
- One riven per weapon: generally known in-game rule, **no wiki statement found** (UNVERIFIED as sourced).

Weapon data fields that drive slot rules (wiki `Module:Weapons/data/*`): `Slot`, `Class` (Rifle 101, Shotgun 38,
Bow 18, Sniper Rifle 15, Launcher 12, Speargun 7, Crossbow 5, Exalted Weapon 4 in primary; 29 melee classes),
`Polarities` (pre-installed general-slot polarities), `ExilusPolarity` (primary: Madurai 95 / Naramon 94; secondary:
Madurai 78 / Naramon 64; melee: `None` on 249 of 250), `StancePolarity`, `MaxRank` (30 or 40),
`CompatibilityTags`, `Family`, `Traits`. The `Exilus Mods` page quotes the U26 dev note "we're making these Slots
default Polarities, meaning all Exilus Weapon Mods are - (Naramon) or V (Madurai) Polarity", consistent with
`ExilusPolarity`.

---

## 2. Mod capacity, drain, polarity, forma

### 2.1 Capacity

| Rule | Value | Source |
|---|---|---|
| Base capacity | = item rank; max rank 30 → 30 | `Mod` § Installation ("Items have a limited Mod Capacity, that correlates to their Rank") |
| Orokin Catalyst (weapons, incl. archguns, archmelee, sentinel weapons) / Orokin Reactor (frames, companions…) | ×2 → 60 at rank 30 | `Orokin Catalyst` § Using ("doubles the weapon's mod capacity … capacity of 20 [unranked, MR 10] … 60 at rank 30") |
| Overlevelled weapons (Kuva, Tenet, Coda, Paracesis; also the Kuva Ayanga and Kuva Grattler archguns, Thermian RPG in `misc`) | each Forma raises max rank by 2, up to 40 after 5 polarizations; capacity 40, **80 with Catalyst** | `Weapons` § Overleveling (lines 21-27). Wiki `MaxRank = 40`: 24 primary, 14 secondary, 12 melee (incl. Paracesis), 4 archwing, 1 misc. |
| Stance mod capacity | stance fully ranked gives +5; matching polarity ×2 = +10; non-matching reduced (see 2.3) | `Stance` § Notes ("All Stances provide a bonus mod capacity of 5 when maxed, doubling it to 10 when placed on the matching polarity"). PE stance `baseDrain` is −2 with max rank 3 → −5. |
| Aura | same mechanism on warframes | `Aura` intro |
| Minimum capacity (unranked gear) | `Mod` page: 15 + 1 per 2 MR (+1 per Legendary Rank after MR 30); `Polarity` page: "minimum capacity is equal to the player's Mastery Rank (up to 30)" | **contradiction**; irrelevant for a max-rank build calculator. |

### 2.2 Drain per rank

`drain(rank) = BaseDrain + rank` — every rank table checked follows it: Serration 4→14 (rank 0→10), Galvanized
Chamber 6→16, Condition Overload 10→15, Critical Delay 4→9, Sacrificial Steel 6→16 (pages `Serration`,
`Galvanized_Chamber`, `Condition_Overload`, `Critical_Delay`, `Sacrificial_Steel`, emodtable "Cost" columns).
`BaseDrain` and `MaxRank` are in wiki `Mods/data` and PE `ExportUpgrades` (`baseDrain`, `fusionLimit`).
Aura/stance: negative base drain (adds capacity).

### 2.3 Polarity modifier

| Slot vs mod polarity | Drain | Source |
|---|---|---|
| Matching | `ceil(drain / 2)` — Serration 14 → 7 | `Mod` § Installation; `Polarity` intro (both agree) |
| Slot unpolarized | `drain` | — |
| Non-matching | **`Mod` page:** "increases drain by a quarter, rounded mathematically: drain 0-1 +0, 2-5 +1, 6-9 +2, 10-13 +3, 14-16 +4" → `floor(drain × 1.25 + 0.5)`. **`Polarity` page:** "+25%, rounded up" → `ceil(drain × 1.25)` | **contradiction**. They differ for drain ≡ 1 mod 4 (1, 5, 9, 13, 17): e.g. Split Chamber 9 → 11 (Mod page) vs 12 (Polarity page). The Mod page gives an explicit bracket table, so it is the more specific source; still UNVERIFIED in-game. |
| Universal "Any" slot (Omni Forma, Aura Forma, Stance Forma) | matches every polarity **except Umbra** | `Polarity` intro list ("Any - O (Any polarity except Umbra)") |
| Umbra slot | halves Umbra mods. Other mods in an Umbra slot: not documented (UNVERIFIED; treat as non-matching until verified) | `Umbra Forma`, `Umbra Mods` |
| Aura/Stance bonus, matching | ×2 | `Mod`, `Polarity`, `Aura`, `Stance` |
| Aura/Stance bonus, non-matching | `Aura` page: "80% of listed drain, rounded down (5 → 4, 9 → 7)"; `Mod` page: "reduces it by 25%, rounded mathematically"; `Polarity` page: "reduce it by 20%" | three wordings; for a max-rank stance (5) all give **4**, so weapons are unaffected. |

Polarities: Madurai, Naramon, Vazarin, Zenurik, Unairu, Penjaga, Umbra, plus the universal "Any" (`Polarity`
intro). Rivens roll Madurai, Naramon or Vazarin (`Riven Mods` intro). Wiki `Mods/data.Polarity` values; PE uses
`AP_ATTACK` (Madurai), `AP_TACTIC` (Naramon), `AP_DEFENSE` (Vazarin), `AP_POWER` (Zenurik), `AP_WARD` (Unairu),
`AP_PRECEPT` (Penjaga), `AP_UMBRA`, `AP_UNIVERSAL` (no polarity), `AP_ANY` (`data-source-public-export-and-connector.md` §1.3).

Forma rules (`Polarity` § Polarization, `Forma`, `Umbra Forma` § Notes, `Stance Forma`):
- Any slot, including pre-polarized ones and the (unlocked) exilus slot, can be re-polarized; no limit on the number
  of Forma; each Forma resets rank to 0 (irrelevant for a calculator).
- Umbra Forma: **only Warframes and melee weapons** (plus Deconstructor/Deconstructor Prime, the only companion
  weapons that can use Umbra mods). Not on primaries, secondaries, archguns. Not directly on a warframe aura/exilus slot.
- Stance Forma: universal polarity on a melee stance slot only. Omni Forma: universal polarity on any other slot.
- Vinquibus: primary and melee forms share Forma (`Umbra Forma` § Notes line 67) — one polarity layout for two weapons.

### 2.4 Worked example (capacity check)

Braton Prime (`Polarities = nil`, `ExilusPolarity = Naramon`), Catalyst, rank 30 → capacity 60. Max-rank drains
from `Mods/data`: Serration 14 (Madurai), Galvanized Chamber 16 (Madurai), Primed Bane of Grineer 14 (Madurai),
Point Strike 7 (Madurai), Vital Sense 7 (Madurai), Hellfire 11 (Naramon), Infected Clip 9 (Naramon), Hunter
Munitions 9 (Madurai).
- Unpolarized: 14+16+14+7+7+11+9+9 = **87 > 60** → does not fit.
- 3 Madurai Forma on Serration/Galvanized Chamber/Primed Bane (7+8+7) and 1 Naramon Forma on Hellfire (6):
  7+8+7+7+7+6+9+9 = **60 ≤ 60** → fits with 4 Forma.
- Same layout but Hellfire placed in a Madurai slot: 11 → 14 (both rounding rules) → 68 > 60.

### 2.5 Must the optimizer respect capacity?

In game, a build that exceeds capacity cannot be equipped, so a "true" build must fit. But capacity is almost always
solvable with enough Forma (each slot can be given any polarity), so capacity is a *cost* (Forma count) rather than
a hard limit, except where polarity cannot help: (a) Umbra mods on non-melee weapons (no Umbra Forma),
(b) a fixed exilus polarity the user does not want to change, (c) shared layouts (Vinquibus). The inspiration does
not appear to model capacity or polarity at all (no hits for capacity/polarity/forma in `guide.txt`; the
optimizer notes only mention slot counts). Recommendation for the spec (not a game fact): optional constraint with
three modes — ignore / fit the user's current polarities / fit with at most N additional Forma.

---

## 3. Mod ranks and per-rank values

- Max rank (`MaxRank` / PE `fusionLimit`) is 3, 5 or 10 for weapon mods (Primed, Galvanized, Umbra: 10; Flawed: 3).
- Per-rank value is linear in `(rank + 1)`: `value(r) = value(max) × (r + 1) / (MaxRank + 1)`. Checked on the
  rank tables of Serration (+15% … +165%), Primed Pressure Point (+15% … +165%), Condition Overload (+13.3% …
  +80%), Critical Delay (+33.3%/−3.3% … +200%/−20%, both stats scale), Galvanized Chamber (+7.3% … +80%, on-kill
  +2.7% … +30%), Galvanized Steel, Galvanized Scope, Sacrificial Steel. For multiplicative faction mods the linear
  part is the bonus: Bane of Grineer x1.05 … x1.30; Flawed Bane x1.03 (=1.025 rounded) … x1.10.
- Displayed lower-rank values are rounded to 1 decimal (80/11 = 7.27 → "+7.3%"); compute from max-rank value.
- Exceptions (non-linear): Sacrificial Steel/Pressure *Sentient* bonus column (x1.02, 1.05, 1.09, … x1.32 —
  not a clean linear series, §8); arcane "+X at max rank only" lines (Primary Merciless +30% reload only at rank 5).
  The `Mods/data` `Description` holds only the max-rank text, so any non-linear mod must be hand-authored.

Worked example: Galvanized Chamber at rank 7: base multishot 80 × 8/11 = 58.18% (page shows +58.2%); on-kill
30 × 8/11 = 21.82% per stack; 5 stacks → 58.18 + 109.09 = 167.27% (page "Max Bonus" +167.2%).

---

## 4. Item compatibility (which mods a weapon may take)

Source: `Mod/Compatibility` (cached `Mod__Compatibility.wiki`).

- Hierarchy: slot (Primary/Secondary/Melee/Archgun/Archmelee/Robotic) → class (Rifle, Shotgun, Pistol, melee
  classes …) → sub-class (Assault Rifle, Bow, Sniper; Launcher, Speargun, Crossbow, Long Bow; Thrown, Tome) →
  family (e.g. Burston) → specific variant (Burston Prime-only mods like Gilded Truth). A mod's `Type` in
  `Mods/data` names the node it attaches to (Primary, Rifle, Assault Rifle, Bow, Sniper, Shotgun, Pistol, Melee,
  a stance class, a weapon family, or a specific weapon). `Module:Mods/itemCompatibility/data` maps those names to
  internal paths (e.g. "Assault Rifle" → `/Lotus/Weapons/Tenno/Rifle/LotusAssaultRifle`, "Bow" →
  `/Lotus/Weapons/Tenno/Bows/LotusBow`), many entries empty.
- Bows, snipers, launchers etc. are Rifles and take Rifle mods; shotguns are not Rifles. Kitguns inherit the class of
  their chamber; zaws inherit the melee class of their strike. Exalted Artemis Bow takes Bow mods, Neutralizer takes
  Sniper mods, Noctua takes Tome mods, Arquebex Archgun mods, Ironbride Archmelee mods.
- Hidden tags: a mod's `CompatibilityTags` must all be present on the weapon (e.g. Sinister Reach needs `BEAM`), and
  none of its `IncompatibilityTags` may be (e.g. Split Flights excludes `CRPBOW, CROSSBOW, GRNBOW, POWER_WEAPON,
  INFBOW, OMICRUS` → Lenz, Kuva Bramma, exalted weapons, Proboscis Cernos, Nataruk). Tags on mods
  (`Mods/data`): `POWER_WEAPON` 62 (excludes exalted weapons), `SENTINEL_WEAPON` 45, `HOUND_WEAPON` 34,
  `SINGLESHOT` 14, `AOE` 9, `POWER_WEAPON_LITE` 8, `NO_HEAVY_ATTACK` 8, … Tags on weapons (`Weapons/data`):
  `PROJECTILE` 175, `ASSAULT_AMMO` 114, `AOE` 71, `BEAM` 43, `SINGLESHOT` 39, `BATTERY` 34, `POWER_WEAPON` 33,
  stance tags (`SWORDS_STANCE` …), `SECONDARYSHOTGUN` 15, `MODULAR_GUN` 6, etc.
- Rule as an algorithm (inferred from the page, exact semantics of multi-tag `CompatibilityTags` UNVERIFIED):
  `allowed(mod, weapon) = type_node(mod) is an ancestor-or-self of weapon in the hierarchy
  ∧ (mod.CompatibilityTags ⊆ weapon.CompatibilityTags) ∧ (mod.IncompatibilityTags ∩ weapon.CompatibilityTags = ∅)`.

---

## 5. Mutual incompatibility

### 5.1 Game rule

`Mod/Compatibility` § Mod Incompatibility: "Two mods with the same hidden parent type or [a] parent/child
relationship cannot both be equipped on an item at the same time." Example: Serration and Amalgam Serration both
inherit `/Lotus/Types/Game/LotusArtifactUpgrades/BaseArtifactUpgrade`. Families therefore are: normal + Flawed +
Primed + Umbra + Amalgam + Galvanized + Archon variants of one standard mod, plus listed oddities:

- Critical Delay ⟂ Point Strike (corrupted crit mods are the exception to "nonstandard mods stack", `Mod` §
  Nonstandard Mods); Critical Deceleration inherits Blunderbuss; Creeping Bullseye ⟂ Pistol Gambit/Primed Pistol Gambit.
- Berserker Fury ⟂ Fury/Primed Fury.
- Split Flights ⟂ Split Chamber / Galvanized Chamber (bows).
- Argon Scope ⟂ Galvanized Scope.
- Sacrificial Steel ⟂ True Steel; Sacrificial Pressure ⟂ Pressure Point / Primed Pressure Point / Amanata Pressure.
- Spectral Serration and Higasa Serration ⟂ Serration.
- Counter-examples that **do** stack: Charged Chamber + Primed Chamber ("Primed" here is not a Primed mod), Heavy
  Caliber + Serration, Spoiled Strike + Pressure Point, Vigilante Armaments + Split Chamber, Augur Pact + Hornet Strike.

### 5.2 Data encoding

- Only wiki `Module:Mods/data` encodes it: field `Incompatible = { "Amalgam Serration", "Higasa Serration",
  "Flawed Serration", "Spectral Serration" }` (475 of 1550 mods). The parent type itself is **not** in any source.
  WFCD `Mods.json` and PE `ExportUpgrades` have no incompatibility field (grep for incompat/compatib/exclu/parent:
  only `excludeFromCodex`).
- The relation is a hand-maintained list of names, **not transitive-closed and not grouped**. Symmetry check over all
  mods: every target name exists; only 3 asymmetric pairs, none on primary/secondary/melee weapons:
  Primed Electrified Barrel → Electrified Barrel (archgun; reverse missing), Primed Streamline → Streamline / Flawed
  Streamline (warframe). Implementation: treat as an undirected graph (union of both directions).

---

## 6. Conditional and stacking mods

### 6.1 Galvanized mods (all max rank 10, Arbitrations vendor)

Source: `Galvanized Mods`, individual pages, `Mods/data` descriptions.

| Mod (class) | Passive at max | On-kill stack (max) | Max stacks | Duration | Notes |
|---|---|---|---|---|---|
| Galvanized Chamber (Rifle) | +80% Multishot | +30% Multishot | 5 | 20 s | max +230% |
| Galvanized Diffusion (Pistol) | +110% Multishot | +30% | 4 | 20 s | max +230% |
| Galvanized Hell (Shotgun) | +110% Multishot | +30% | 4 | 20 s | max +230% |
| Galvanized Aptitude (Rifle) | +80% Status Chance | +40% Direct Damage per status type on target | 2 | 20 s | CO-type |
| Galvanized Savvy (Shotgun) | +80% Status Chance | +40% per status type | 2 | 20 s | CO-type |
| Galvanized Shot (Pistol) | +80% Status Chance | +40% per status type | **3** | **14 s** | CO-type |
| Galvanized Scope (Rifle) | on weak-point hit: +120% CC when aiming (12 s) | on weak-point kill: +40% CC when aiming | 5 | 12 s, **each stack own timer** | max +320% |
| Galvanized Crosshairs (Pistol) | same as Scope | +40% | 5 | 12 s, each stack own timer | |
| Galvanized Acceleration (Shotgun, exilus) | +30% projectile speed and beam range | +30% | 2 | 10 s | |
| Galvanized Steel (Melee) | +110% CC (x2 heavy) | +30% Critical Damage | 4 | 20 s | max +120% CD |
| Galvanized Elementalist (Melee) | +80% Status Damage | +30% Status Chance | 4 | 20 s | |
| Galvanized Reflex (Melee) | +50% Heavy Attack Efficiency | +20 Initial Combo | 4 | 20 s | |

Stacking semantics (`Galvanized Mods` § Notes; `Galvanized Shot` § Notes): only kills by the weapon carrying the mod
give stacks (status kills count, incl. Hunter Munitions slash); the buff only affects that weapon; on timeout
**one** stack is lost and the timer resets (exceptions: Scope, Crosshairs — independent per-stack timers). If the
same Galvanized mod is on a normal weapon and an exalted weapon, both benefit (some exalted weapons keep separate
stacks: Shattered Lash). Robotic weapons get stacks only from the player's kills with a weapon carrying the same mod.
Galvanized Steel/Elementalist: stacks gained with archmelee but bonus does not apply (bug).

CO-type timing and per-weapon additive/multiplicative behaviour: see `wiki-additive-vs-multiplicative-edge-cases.md`
§1 (new status benefits only later hits; multishot pellets act as sequential hits).

### 6.2 Other conditional weapon mods (max-rank values from `Mods/data`)

| Mod | Effect | Condition / stacking |
|---|---|---|
| Condition Overload (Melee) | +80% melee damage per status type | target statuses; see edge-cases file |
| Weeping Wounds (Melee) | +40% status chance per combo multiplier | combo |
| Blood Rush (Melee) | crit chance per combo multiplier (not matched by my regex; value on its page) | combo |
| Berserker Fury (Melee) | +35% attack speed on melee kill, 10 s | stacks 2x |
| Argon Scope (Rifle) / Hydraulic Crosshairs (Pistol) / Laser Sight (Shotgun) | +135% / +135% / +120% CC when aiming, 9 s | on weak-point hit |
| Bladed Rounds (Rifle) / Sharpened Bullets (Pistol) / Shrapnel Shot (Shotgun) | +120% / +75% / +99% crit damage when aiming, 9 s | on kill |
| Catalyzer Link / Embedded Catalyzer / Nano-Applicator | +60% / +90% / +90% status chance when aiming, 9 s | on ability cast |
| Spring-Loaded Chamber / Pressurized Magazine / Repeater Clip | +75% / +90% / +105% fire rate when aiming, 9 s | on reload |
| Charged Chamber (Sniper) / Primed Chamber (Sniper) | +40% / +100% damage | first shot in magazine; the two stack |
| Split Flights (Bow, non-AoE) | +100% multishot, −180% accuracy, 2 s | on hit, stacks 4x |
| Spectral Serration (Rifle) | +330% damage | while invisible |
| Motus Setup (Shotgun) | +100% crit and status chance, 4 s | after double/bullet jump landing |
| Mark of the Beast (Thrown Melee) | +120% status and crit chance for secondary, 24 s | 6 melee kills within 6 s |
| Combo Fury (Thrown Melee) | +100% reload, +100% magazine on secondary, 12 s | on melee kill |
| Deadly Efficiency (Archgun) | +120% damage, 9 s | on reload from empty |
| Critical Focus / Marked Target (Archgun) | +60% CC and CD / +120% SC | when aiming |
| Killing Blow, Amalgam Organ Shatter, Melee Elementalist (melee) | heavy-attack-only effects ("Melee Damage On Heavy Attack", wind-up) | attack type |
| Synth Charge (Pistol, set) | +200% damage on final shot of magazine | magazine ≥ 6 |
| Proton Snap (Melee) | +100% toxin, +50% status chance, 20 s | hold wall latch 2 s |
| Tennokai mods (Master's Edge +60% Tennokai damage, Truth's Flame +120%, Dreamer's Wrath +32% CD, Condition's Perfection +100% SC) | exilus melee mods | Tennokai attacks only |

The calculator needs an explicit condition model per mod: `{trigger, duration, max_stacks, decay: one|all|per_stack,
requires: aiming|weakpoint|invisible|heavy|first_shot|final_shot|combo|status_count}`; none of the data sources
provides it (§11).

---

## 7. Set mods

### 7.1 Counting rules

- Set bonus strength depends only on the **number of different set mods equipped**, not on their rank (`Set Mods`
  intro: "Set Bonus effects are not improved by upgrading the mod through regular fusion").
- Since **U38.5**: "Set Bonuses are now calculated based on the Active Weapon" — count = set mods on the Warframe +
  on the *active* weapon (+ companion mods, see below). Mods on the holstered weapons do not count. Exalted weapons
  now contribute/benefit (`Set Mods` § Patch History U38.5).
- Set mods on Sentinel/MOA robotic weapons count; the same set mod on both the player's weapon and the robotic
  weapon counts once (`Set Mods` § Notes). Melee robotic weapons cannot equip set mods.
- U41 fixed Gladiator, Vigilante, Carnis, Mecha, Nira, Saxum, Synth and Tek set bonuses "not applying" — data from
  before U41 (2025?) may reflect broken behaviour.
- Single set mods already give the 1-piece bonus.

### 7.2 DPS-relevant sets

| Set (pieces) | Per-piece stats relevant to weapons | Set bonus by count | Source |
|---|---|---|---|
| Vigilante (6: Armaments +60% MS, Fervor +45% FR (x2 bows), Offense +1.5 PT, Supplies [primary exilus], Pursuit [frame], Vigor [frame]) | as listed | "Chance to enhance critical hits" 5/10/15/20/25/30% for 1–6 | `Vigilante Set`; PE ExportModSet: "30% chance to enhance Critical Hits from Primary Weapons" |
| Gladiator (6: Might +60% CD, Vice +30% AS, Rush +6 s combo duration [melee]; Aegis, Finesse, Resolve [frame]) | | +10% melee crit chance per combo multiplier **per piece** (60% at 6) | `Gladiator Set`; PE text "10% … 60% Melee Critical Chance per Combo Multiplier" |
| Hunter (6: Munitions [primary] +30% chance to apply Slash on crit, Track [primary] +90% status duration; Adrenaline [frame]; Command, Recovery, Synergy [companion]) | Munitions | +25% companion damage vs Slash-affected enemies per piece (150% at 6) — companion only | `Hunter Set` |
| Augur (6: Pact +90% damage, Seeker +90% status duration [pistol]; 4 frame mods) | Pact | energy→shields 40% per piece — no DPS | `Augur Set` |
| Synth (4: Charge [pistol] +200% damage on final shot; 2 companion, 1 frame) | Charge | 5% magazine reloaded per piece when holstered | `Synth Set` |
| Mecha (4: Empowered [aura] "squad deals +150% extra damage against a Marked enemy"; Overdrive [kubrow claws]; Pulse [frame]; Recharge [kubrow]) | Empowered | mark every 60/45/30/15 s, mark duration 3/6/9/12 s, status spread 7.5/15/22.5/30 m; needs Kubrow or Predasite | `Mecha Set` |
| Tek (4: Collateral [frame] "+100% Critical Damage when inside the Marked Zone"; Gravity [melee]; 2 kavat) | Collateral | Marked Zone radius 3/6/9/12 m, cooldown 60/45/30/15 s, 50/100/150/200 Slash dmg/s | `Tek Set` |
| Strain (4: Infection [melee] +20% crit damage per Cyst; Fever, Eruption [Helminth charger]; Consume [frame]) | Infection | Cysts 2/4/6/8, duration 6/12/18/24 s | `Strain Set`. Max Infection value 20% × 8 = 160% — derived, UNVERIFIED |
| Sacrificial (2 melee: Steel +220% CC (x2 heavy), Pressure +110% melee damage; both x1.33 vs Sentients) | | with both equipped, each mod's values ×1.25 (Steel 275% CC, Pressure 137.5%) | `Sacrificial Set`, `Sacrificial Steel`/`Pressure` rank tables (set level 2 column). PE `modSetValues = [0.25, 0.5]` (index = pieces − 2; 0.5 unreachable). |
| Nira's (3) | | +150% Slam damage per piece (450% at 3) since U39 (was 100%) | `Nira's Set` |
| Carnis / Jugulus / Saxum (3 each: melee + pistol mod +90% Slash/Puncture/Impact and +60% status chance) | the mods | non-DPS (evasion) / tendrils / lifted-enemy explosion | respective pages |
| Umbral (3 frame) | — | 2 pcs: +30%/+25%(Intensify); 3 pcs: +80%/+75% | `Umbral Set`; PE `modSetValues` Intensify [0.25, 0.75], Fiber/Vitality [0.3, 0.8] |

Worked examples:
- Gladiator: 3 Gladiator mods on melee, combo multiplier 12x. Bonus = 10% × 3 × (12 − 1) = +330% to the melee
  crit-chance bucket (page: "Critical Chance at 12x" column = 110/220/330/…; 110 = 10 × 11, so the factor is
  `(combo − 1)`).
- Vigilante: 4 primary + 2 frame pieces → 30%. Weapon CC 150% → tier-1 crit 50%, tier-2 50%. Each crit is raised one
  tier with 30%: P(tier 1) = 0.5 × 0.7 = 0.35, P(tier 2) = 0.5 × 0.3 + 0.5 × 0.7 = 0.50, P(tier 3) = 0.5 × 0.3 = 0.15.
  Expected tier = 1.8 (vs 1.5 without). Whether the enhancement can also turn a non-crit into a crit: the page says
  it applies to critical hits only ("if the weapon deals a yellow critical …").
- Sacrificial Steel rank 10, both set mods: +220% × 1.25 = +275% CC.

---

## 8. Riven mods

Sources: `Riven Mods` (cached `Riven_Mods.wiki`, lines 1-345 ranges tables, 761-1226 attributes and formula,
1227-1470 cycling/locking/splicing, 1515-1537 notes), `Riven Mods/Patch History`, `Update 44: Iceblade of Narin`
(dates), PE `ExportUpgrades` riven records.

### 8.1 Timeline (newest first)

| Update | Date | Change |
|---|---|---|
| Hotfix 44.1.3 | 2026-10-22 | **Riven dispositions removed.** "All Rivens now use the same multiplier … the equivalent of 1.2 Disposition." Old range was 0.5–1.55. Ammo Efficiency capped at 75% globally. Riven trading restored. |
| 44.1.1 | 2026-10-08 | Fixed Weak Point Damage spliced trait not applying its bonus. |
| Update 44.1 (The Icebind) | 2026-10-07 | Riven Splicing introduced (Riven Splicer). |
| 44.0.2 | 2026-09-28 | Shotgun rivens can roll Zoom (needed for splices). |
| Update 44 | 2026-09-23 | Trait locking (one trait, doubles Kuva cost; locks the +/− layout). |

Wiki `Module:Weapons/data` already has `Disposition = 1.2` on every weapon and the old value in
`_RetiredDisposition` (e.g. Paris 1.4, Felarx 0.6, Nataruk 0.65, Kuva Bramma 0.7). The inspiration's guide (§17)
still talks about per-weapon "synergy" (disposition); whether its code still applies old dispositions was not
checked here.

### 8.2 Structure

- Classes: Rifle (non-shotgun primaries), Shotgun, Pistol (secondaries), Melee, Zaw, Kitgun (primary or secondary
  kitguns), Archgun (also usable on atmospheric archguns), Companion Weapon (robotic weapons; base values of the
  associated class, e.g. Deconstructor = Melee, Sweeper = Shotgun). PE riven records (`/Lotus/Upgrades/Mods/
  Randomized/Lotus{Rifle,Shotgun,Pistol,ModularPistol,ModularMelee,Archgun}RandomModRare`,
  `PlayerMeleeWeaponRandomModRare`) carry 36–38 stat entries; **no Companion record** with stats.
- Kitgun rivens use the **Pistol** coefficient column ("Pistol / Kitgun") even on primary kitguns; Zaw uses Melee.
- A riven is bound to one weapon and its variants (MK1, Prime, Vandal, Wraith, Dex, Prisma, Mara, Syndicate). Not
  shared: Mutalist Cernos/Quanta vs Cernos/Quanta, single vs dual sidearms, Dex Dakra vs Dakra Prime, Nikana vs
  Dragon Nikana. No rivens for exalted weapons.
- Rank 0–8; values below are at rank 8. Polarity Madurai/Naramon/Vazarin. MR 8–16.
- Traits: 2 or 3 positive, 0 or 1 negative; a spliced riven can show 4 traits (3+1). Only one spliced trait per riven.
- Drain: not documented on the wiki; PE `baseDrain 9, fusionLimit 0` is the *veiled placeholder* only. The real
  per-riven drain is printed on the card and must be a user input (UNVERIFIED; commonly 10–18 at rank 8).

### 8.3 Value formula (`Riven Mods` § Attribute Value Formula, detailed collapsible)

```
v = b × 10 × (r + 1) × D × L × W          (32-bit float, step by step)
W = 0.9 + 0.2u,  0 ≤ u ≤ 1               (random quality; u = 0.5 → W = 1)
D = 1.2                                   (constant since Hotfix 44.1.3)
r = riven rank 0..8                       (r = 8 → factor 90)
displayed: percentage stats 100v %; Ammo Efficiency and Chance to Gain Combo Count −100v %;
           faction stats ×(1 + v)
```

| Traits | L (each positive) | L (negative) |
|---|---|---|
| 2 positive, 0 negative | 0.99 | — |
| 2 positive, 1 negative | 1.2375 | −0.495 |
| 3 positive, 0 negative | 0.75 | — |
| 3 positive, 1 negative | 0.9375 | −0.75 |

Splice: the spliced trait keeps the **higher random value u** of the two consumed traits (even if that is the
negative's), is always positive, and keeps the riven's +/− layout. Its L presumably follows the riven's
positive/negative count (UNVERIFIED; the page gives no separate rule). The overview tables list spliced ranges in
the same four columns as normal traits, which supports that reading.

Worked examples (rank 8):
- Rifle Critical Chance, b = 0.016666, 2+1: 0.016666 × 90 = 1.49994; × 1.2 = 1.79993; × 1.2375 = 2.22741;
  × W ∈ [0.9, 1.1] → **+200.5% … +245.0%** (overview table: "200.5% — 245.0%").
- Rifle Damage negative, b = 0.0183333, 2+1: 0.0183333 × 90 × 1.2 × (−0.495) = −0.980; × W → **−88.2% … −107.8%**
  (table ✓).
- Faction (any class), b = 0.005, 2+0: 0.005 × 90 × 1.2 × 0.99 = 0.5346 → ×1.535 at W = 1; range **x1.48 … x1.59**
  (table ✓). Negative 2+1: 0.45 × 1.2 × (−0.495) = −0.267 → **x0.71 … x0.76** (table ✓).

### 8.4 Coefficients b (rank formula input; `Riven Mods` § Base Values, collapsible table)

Ordinary traits (Rifle | Shotgun | Pistol/Kitgun | Archgun | Melee/Zaw; "–" = cannot roll):

| Trait | Rifle | Shotgun | Pistol/Kitgun | Archgun | Melee/Zaw |
|---|---|---|---|---|---|
| Damage / Melee Damage (see note below the table) | 0.0183333 | 0.0183 | 0.0244 | 0.0111 | 0.0183 |
| Critical Chance (melee: x2 for heavy) | 0.016666 | 0.01 | 0.016666 | 0.0111 | 0.02 |
| Critical Damage | 0.013333 | 0.01 | 0.01 | 0.0089 | 0.01 |
| Multishot | 0.01 | 0.0133 | 0.0133 | 0.0067 | – |
| Fire Rate (x2 bows) / Attack Speed | 0.00667 | 0.01 | 0.0083 | 0.00667 | 0.0061 |
| Status Chance | 0.01 | 0.01 | 0.01 | 0.0067 | 0.01 |
| Status Duration | 0.01111 | 0.01111 | 0.01111 | 0.01111 | 0.01111 |
| Heat / Cold / Electricity / Toxin (positive only) | 0.01 | 0.01 | 0.01 | 0.0133 | 0.01 |
| Impact / Slash | 0.013333 | 0.013333 | 0.013333 | 0.01 | 0.0133 |
| Puncture | 0.01333 | 0.01333 | 0.01333 | 0.01 | 0.0133 |
| Damage vs Corpus / Grineer / Infested | 0.005 | 0.005 | 0.005 | 0.005 | 0.005 |
| Magazine Capacity | 0.005555 | 0.005555 | 0.005555 | 0.0067 | – |
| Reload Speed | 0.005555 | 0.005555 | 0.005555 | 0.0111 | – |
| Ammo Maximum | 0.00555 | 0.01 | 0.01 | 0.0111 | – |
| Projectile Speed | 0.01 | 0.01 | 0.01 | – | – |
| Punch Through (positive only, metres) | 0.03 | 0.03 | 0.03 | 0.03 | – |
| Weapon Recoil | −0.01 | −0.01 | −0.01 | −0.01 | – |
| Zoom | 0.006666 | 0.004666 | 0.0089 | 0.006666 | – |
| Range (m) | – | – | – | – | 0.02158 |
| Combo Duration (s) | – | – | – | – | 0.09 |
| Initial Combo (positive only) | – | – | – | – | 0.27224 |
| Additional Combo Count Chance (positive only) | – | – | – | – | 0.00653 |
| Chance to Gain Combo Count (negative only) | – | – | – | – | −0.01165 |
| Critical Chance for Slide Attack | – | – | – | – | 0.013334 |
| Finisher Damage | – | – | – | – | 0.0133 |
| Heavy Attack Efficiency | – | – | – | – | 0.00816 |

Note on Damage / Melee Damage: the page's base-values table marks it with footnote ¹ ("positive values only"), but
the overview tables list negative Damage ranges (Rifle 3+1: −133.7% … −163.4%) and the Update 44.1 patch notes
show a "-222.6% Damage" trait on a Hikou riven. The footnote marker contradicts both; treat Damage as able to be
negative. Logged under Open questions.

Spliced traits (always positive): Blast, Corrosive, Gas, Magnetic, Radiation, Viral damage 0.01 (all classes);
Damage vs Orokin / Scaldra / Techrot 0.005; Status Damage 0.01; Weakpoint Damage 0.025 and Weakpoint Critical
Chance 0.0275 (ranged only); Ammo Efficiency −0.005 (ranged); Reload While Holstered 0.01 (Rifle/Shotgun/Pistol);
melee only: Heavy Attack Wind Up Speed 0.0133, Melee Damage On Heavy Attack 0.0133, Slam Attack Damage 0.0133,
Parry Angle 0.009. Recipes (§ Splicing): Heat+Cold→Blast, Toxin+Electricity→Corrosive, Toxin+Heat→Gas,
Electricity+Cold→Magnetic, Heat+Electricity→Radiation, Toxin+Cold→Viral, Corpus+Grineer→Orokin,
Infested+Grineer→Scaldra, Corpus+Infested→Techrot, Damage+Status Chance→Status Damage, Damage+Zoom or
Damage+Multishot→Weakpoint Damage, Crit Chance+Zoom or Crit Chance+Multishot→Weakpoint Crit Chance,
Magazine+Reload or Recoil+Ammo Max→Ammo Efficiency, Ammo Max+Reload or Ammo Max+Magazine→Reload While Holstered,
HAE+Additional Combo Count→Heavy Attack Damage, HAE+Combo Duration→Wind Up, Attack Speed+Range→Parry Angle,
Melee Damage+Attack Speed→Slam Damage.

Sign rules: elemental (Heat/Cold/Electricity/Toxin), Punch Through, Initial Combo, Additional Combo Count Chance are
positive-only; Chance to Gain Combo Count is negative-only; a physical type can only roll if the weapon has > 25%
of it ("usually", exceptions case by case); vintage companion traits (Zoom, Recoil, Ammo Max) cannot be cycled in.

### 8.5 How riven traits enter the damage calc (catalog side)

- They are ordinary stat lines of the corresponding standard mod and stack in the same bucket (Damage with
  Serration, CC with Point Strike …). Riven faction stats are faction damage bonuses (`Faction Damage Bonus` §
  Other Sources: "base value of 45%, then modified according to the attribute value formula").
- Multiple riven elements are applied **bottom-up** for element combination: a mod placed *before* the riven
  combines with the riven's *last* listed element; a mod placed *after* combines with the *first* listed element
  only if the riven has 1 or 3 elements; the same applies to an adversary weapon's valence element (`Riven Mods` §
  Notes). Example from the page: Heat mod before a riven listing Electricity then Cold → Blast + Electricity
  (instead of Magnetic).

---

## 9. Incarnon evolutions

Sources: `Incarnon` § Mechanics, § Incarnon Genesis; per-weapon `<Weapon> Incarnon Genesis` pages (cached Furis,
Braton, Lex, Torid); Update 44.1.3 notes.

- 53 Incarnon weapon lines: 8 native (Zariman: Felarx, Innodem, Laetum, Phenmor, Praedos; Sanctum: Onos, Ruvox;
  Isleweaver: Thalys) + 45 Incarnon Genesis adapters (Ack & Brunt … Zylok; full list in `Incarnon.wiki` lines 38-94).
- Native incarnons: **5** evolutions; Genesis: **4** evolutions (`Incarnon` § Mechanics: "Each weapon has a total of
  4 or 5 evolutions").
- Evolution I: unlocks Incarnon Form (alt-fire or stat change). Remaining evolutions: **2 or 3 selectable perks, exactly
  one per evolution**, changeable freely in the Arsenal. → optimizer variable: one choice per tier.
- Perk values can differ per variant (Furis X = 28 vs MK1-Furis X = 34 base damage; Braton line has four variant
  columns). Perk tables are wikitext only (no module; see `data-source-wiki-modules.md`).
- Genesis applies to all variants (Prime/Wraith/Vandal) but not to Akimbo/Dual versions.
- Incarnon Form activation: primary/secondary charge by weak-point hits (Angstrum, Torid: any direct hit); form uses
  its own ammo (gauge). Melee: 6x combo (Innodem 5x) + heavy attack, lasts 180 s (90 s Innodem, Ruvox, Obex).
- Perk semantics relevant to the calculator (from Genesis pages):
  - "Increase Base Damage by +X": flat addition to base damage (bucket `X` in the edge-cases file). Known bug: Braton
    line, Galvanized Aptitude's per-status bonus does not apply to the +X part (`Braton_Incarnon_Genesis.wiki`
    Known Bugs).
  - "Increase Base Critical Damage Multiplier by +3x when CC < 40%" (Prelude of Might): added to *base* CDM, then
    modded; the < 40% check uses modified CC including Puncture-status and Covenant/Galvanized Scope/Primary Acuity bonuses.
  - Torid "Plentiful Mayhem": +60% damage on multishot-generated projectiles, multiplicative with Serration; in
    Incarnon Form it instead raises all multishot bonuses by 60%.
  - Extended Volley-type magazine perks do not apply to the Incarnon Form.
  - Ammo-efficiency perks were rebalanced in 44.1.3 to respect the 75% cap (Bronco "Enough for Everyone" 80→75%,
    Dera/Boltor "Crimson Overture" now +25% × 3 stacks).

---

## 10. Kuva / Tenet / Coda (adversary) weapons

Sources: `Kuva Weapons`, `Tenet Weapons`, `Coda Weapons` § Elemental Bonus, `Valence Fusion`, `Weapons` §
Overleveling, `Damage` § Elemental Damage Combinations, wiki `Warframes/data` field `Progenitor`.

- Valence bonus: one of **Impact, Heat, Cold, Electricity, Toxin, Magnetic, Radiation**, **25–60% of the weapon's base
  damage**, added as base damage ("applies as weapon base damage, meaning elemental mods and status that scale from
  base/modified base damage will be affected"). Element: Kuva/Tenet from the progenitor warframe (wiki
  `Warframes/data` `Progenitor` field); Coda and store-bought Tenet: random at shop refresh. Elemental Vice at rank 40
  allows free switching (amount unchanged).
- Distribution: Kuva page: uniform since **U42** ("now properly randomized … equal chance across the bonus range");
  Tenet and Coda pages still say "lower percentages being more common" (citing a 2019 Reddit post) — **outdated**
  relative to the U42 note for Kuva/Tenet; Coda not mentioned in that note.
- Valence Fusion: `new = min(1.1 × max(a, b), 60%)`, rounded **down** to 0.1%; results ≥ 58% become 60% (U36).
  Example: 52% fused with 40% → 57.2%; 52.8% → 58.08 → 60%. Max is reached from 25% in at most 9 fusions.
- Overleveling: max rank 40 after 5 Forma (+2 per Forma), capacity 40 / 80 with Catalyst. Paracesis also gains
  "up to 60% faction damage bonus against Sentients" from ranks above 30 (`Faction Damage Bonus` § Other Sources).
- Element ordering: innate elements go last in the combination order; when a Kuva/Tenet weapon has two innate primary
  elements (weapon's own + valence), the one earlier in **Heat > Cold > Electricity > Toxin** is placed second-to-last
  and the other last (`Damage` § Elemental Damage Combinations).
- Impact valence: a physical type that "scales like" a base-damage addition (the bonus is a % of total base damage,
  not of existing Impact).

Worked example: Kuva Hek-like attack with 400 total base damage and 48% Toxin valence → +192 Toxin added to base
damage (total 592), then Serration etc. multiply the 592.

---

## 11. Weapon arcanes

Sources: `Arcane Enhancement` (slots, ranks, categories), `Arcane/data` module (166 entries), individual arcane
pages (Primary Merciless), `Archgun`.

- Max rank: 5 for Warframe, Primary, Secondary, Melee, Operator, Eternal arcanes; **3** for Exodia (zaw), Pax and
  Residual (kitgun), Virtuos (amp) (`Arcane Enhancement` § Rank table). Rank r needs (r+1)(r+2)/2 copies.
- Per-rank value: linear in (r+1) like mods where checked (Primary Merciless 5/10/15/20/25/30% per stack); some
  lines exist only at max rank (Merciless +30% reload at rank 5 only; "+1 Arcane Revive" on warframe arcanes).
- Arcane types in `Arcane/data.Type`: Primary 14, Secondary 18, Melee 12, Kitgun 8, Zaw 8, Bow 1 (Longbow
  Sharpshot), Shotgun 1 (Shotgun Vendetta), Warframe 64, Operator 22, Amp 13, Tektolyst Artifacts 5. Bow/Shotgun/
  Akimbo/Conjunction/Cascadia arcanes go in the primary/secondary slot but their effect is limited to that weapon
  type (e.g. "Akimbo Slip Shot … with Dual Pistols").
- Primary arcane effects do not apply to secondaries and vice versa (§ Notes). On archguns, primary/secondary-weapon
  arcanes are allowed in the two slots but class-restricted *warframe* arcanes (Arcane Precision etc.) do not apply.
- Stacking arcanes (max rank, `Arcane/data` descriptions):

| Arcane | Effect at max rank | Stacks / duration |
|---|---|---|
| Primary / Secondary Merciless | +30% damage on kill | 12x, 4 s, lose one stack per timeout (48 s full decay); +30% reload passive |
| Primary / Secondary Deadhead | +120% damage on weak-point kill; +30% headshot multiplier; −50% recoil | 3x, 24 s |
| Primary / Secondary Dexterity | +60% damage on melee kill; +7.5 s combo duration | 6x, 20 s |
| Primary Frostbite | +3% CD and +2.25% MS per Cold status | 40x, 12 s |
| Primary Blight | +3.6% CD and +1.8% MS per Toxin status | 40x, 12 s |
| Conjunction Voltage | +1.5% reload and +3% MS per Electricity status | 40x, 12 s |
| Primary Crux | +30% SC, +6% ammo efficiency on weak-point hit | 10x, 10 s |
| Cascadia Flare | +12% damage per Heat status, 10 s | up to 480% (40 stacks) |
| Melee Exposure | +60% Corrosive on ability cast, 25 s | up to 240% (4x) |
| Melee Animosity | +42% CC on next heavy attack per melee hit | up to 420% |
| Melee Retaliation | +30% melee damage per 200 shields | up to 420%, halved for overshields |
| Primary Bulwark | +1% damage per armor above 1000 | up to +500% |
| Primary Overcharge | 35% of max energy as multishot while ≥ 90% energy | cap 350% |
| Primary Compression | aiming: x0.2 explosion radius, +100% damage per 1 m radius lost | — |
| Secondary Outburst | consume combo: +20% CC and CD per combo consumed, 30 s | — |
| Secondary Kinship | +20% CC per ally-warframe buff | — |
| Secondary Enervate | +10% CC per hit, resets after 6 big crits | — |
| Secondary Shiver | enemies take +45% damage per Cold status | — |
| Longbow Sharpshot | +300% damage on next shot after weak-point hit (bows) | — |
| Shotgun Vendetta | +180% MS, +75% reload for 15 s on kill within 5 m | — |
| Melee Duplicate | on base critical hits: 100% chance to strike a second time | — |
| Melee Doughty | +1.0x CD per 10% Puncture status chance | — |
| Melee Careen | x2.5 melee damage vs frozen | — |

- Warframe arcanes that buff weapons (2 slots, different arcanes only): Arcane Avenger (+45% *absolute* CC, 21% on
  damaged, 12 s), Arcane Fury (+180% melee damage), Arcane Strike (+60% AS), Arcane Blade Charger (+300% melee),
  Arcane Primary Charger (+300% primary), Arcane Rage (+180% primary on headshot, 15%), Arcane Rise (+150% primary on
  reload, 60%), Arcane Awakening (+150% pistols), Arcane Precision (+300% secondary on weak-point hit), Arcane
  Acceleration (+90% FR primary, excl. shotguns), Arcane Velocity (+120% FR pistols), Arcane Tempo (+90% FR shotguns),
  Arcane Momentum (sniper reload), Arcane Arachne (+150% primary/secondary on wall latch), Arcane Hot Shot (+6% CC
  per stack, 50x, from ability heat procs), Arcane Crepuscular (+3x final crit damage while invisible). Their bucket
  placement is in the edge-cases file and `Universal Weapon Bonuses`.

---

## 12. Machine-readable sources for mods and arcanes

| Need | wiki `Module:Mods/data` | wiki `Module:Arcane/data` | WFCD `Mods.json` / `Arcanes.json` | DE Public Export `ExportUpgrades` / `ExportRelicArcane` / `ExportModSet` |
|---|---|---|---|---|
| Identity (uniqueName) | `InternalName` (1,451/1,635 match PE) | `InternalName` (161/166; 5 "Zid-An" Antiques not in PE) | `uniqueName` (copied from PE) | **authoritative** |
| Polarity, rarity, base drain, max rank | yes | rarity, `MaxRank` | yes | yes (`AP_*`, `baseDrain`, `fusionLimit`) |
| Compatibility class | `Type` (262 values) + `CompatibilityTags` / `IncompatibilityTags` | `Type` | `compatName` (coarse) | `compatName` (coarse) |
| Mutual exclusion | **`Incompatible` (only source)** | n/a | none | none |
| Exilus flag | `IsExilus` (96 non-frame; one oddity: Flawed Ravage flagged, UNVERIFIED) | n/a | `isExilus`? not checked | not checked |
| Set membership / bonus | `Set`; `Sets` table (prose at max count, `NumUpgradesInSet`) | n/a | `modSet`, `modSetValues` | `modSet`; `ExportModSet.stats[]` = **bonus text per piece count** (1..N); numeric `modSetValues` only for Umbral/Sacrificial |
| Effect values | max-rank prose `Description` + `UpgradeTypes` enum (no values) | max-rank prose + `UpgradeTypes` (sometimes wrong) | `levelStats[r].stats[]` per rank, text with `<DT_*>` tokens | same `levelStats` text per rank |
| Conditional effects | prose: "On Kill:\r\n+30% Multishot for 20s. Stacks up to 5x." | prose; `IsRefreshable` flag | prose per rank | prose per rank |
| Riven stat coefficients | none (stub `Class = "Riven"`) | — | `upgradeEntries` copied | **`upgradeEntries`: tag, prefix/suffix, `value` = b** (Rifle CC 0.016666 → ×90×1.2×L) |
| Incarnon perks | `DefaultUpgrades` ids only (empty descriptions) | — | none | none |

Conclusion: no source has machine-usable conditions or stacking. Build the catalog as: PE (identity, drain,
polarity, rank, riven coefficients, set text per count) + wiki `Mods/data` (compatibility tags, `Incompatible`,
`IsExilus`, `Type`) + a hand-authored effects file (stat bucket, value at max rank, condition model from §6),
regression-checked against PE `levelStats` text at every rank. Incarnon perks and set bonuses need hand-authoring
from the wikitext pages.

---

## Edge cases

1. Non-matching polarity rounding: `Mod` page (round half up) vs `Polarity` page (round up) differ for drains 1, 5, 9, 13, 17 (e.g. 9 → 11 vs 12).
2. Aura/stance non-matching bonus: 80% floor (Aura) vs 75% round (Mod) vs −20% (Polarity); all give 4 for a max stance.
3. Umbra Forma only on warframes, melee weapons and Deconstructor (Prime); Umbra mods on primaries can never be halved.
4. "Any" (Omni/Stance/Aura Forma) polarity matches everything except Umbra.
5. Vinquibus shares one Forma/polarity layout between its primary and melee forms.
6. Overlevelled weapons (Kuva/Tenet/Coda/Paracesis, incl. Kuva Ayanga and Kuva Grattler archguns) have 40/80 capacity, not 30/60.
7. Melee weapons have an exilus slot since U35; the `Exilus Mods` page still says primary/secondary only.
8. Archguns: 2 arcane slots (one primary-arcane, one secondary-arcane), but class-limited warframe arcanes do not apply.
9. Kitgun/Zaw: kitgun (Pax/Residual) or zaw (Exodia) arcane slot is in addition to the normal primary/secondary/melee arcane slot.
10. Kitgun rivens use Pistol coefficients even on primary kitguns; robotic rivens use the class of the associated weapon.
11. Riven dispositions retired in Hotfix 44.1.3 (2026-10-22); any data with per-weapon dispositions is outdated (old values kept in `_RetiredDisposition`).
12. Riven element order is bottom-up; mods before/after the riven combine with different riven elements.
13. Spliced riven trait inherits the higher u of the two consumed traits; at most one spliced trait; spliced riven may have 4 traits.
14. Riven Damage is footnoted "positive only" in the base-values table but has negative ranges in the overview tables and patch notes.
15. Physical riven traits only roll if the weapon has > 25% of that type ("usually").
16. Charged Chamber + Primed Chamber stack (Primed Chamber is not a Primed variant); Critical Delay vs Point Strike do not.
17. `Incompatible` lists are not symmetric in 3 cases (Primed Electrified Barrel, Primed Streamline); union both directions.
18. Galvanized Scope/Crosshairs stacks expire individually; other Galvanized mods lose one stack per timeout.
19. Galvanized Shot: 3 stacks for 14 s vs Aptitude/Savvy 2 stacks for 20 s.
20. Galvanized Aptitude's GunCO bonus ignores Braton-line Incarnon "+Base Damage" perks (known bug).
21. Set bonus counts only Warframe + *active* weapon (+ companion) since U38.5; duplicate set mods on player and robotic weapon count once.
22. Gladiator set bonus scales with `(combo − 1)` and per piece; Nira's set changed to 150%/piece in U39.
23. U41 fixed several set bonuses not applying (Gladiator, Vigilante, Synth, Tek, Mecha …).
24. Sacrificial set bonus ×1.25 applies to the primary stat; the Sentient bonus column is non-linear and inconsistent with "41.3%".
25. Primary Merciless reload bonus exists only at max rank (non-linear per-rank).
26. Valence bonus distribution changed to uniform in U42 (Kuva/Tenet); Tenet/Coda pages still say "lower more common".
27. Two innate primary elements on adversary weapons are ordered by Heat > Cold > Electricity > Toxin.
28. Incarnon perk values differ by weapon variant (Furis vs MK1-Furis).
29. Incarnon magazine perks do not apply to Incarnon Form; Torid's Plentiful Mayhem changes meaning inside the form.
30. Ammo Efficiency capped at 75% from all sources since Hotfix 44.1.3.
31. `IsExilus` flags Flawed Ravage (a crit-damage mod) as exilus — likely data error.
32. Pre-U41 inspiration or community data may encode set bonuses that were bugged at the time.

## Open questions

1. Non-matching drain rounding: round-half-up (Mod page) or ceil (Polarity page)? Needs an in-game check with a 9-drain mod in a wrong slot.
2. What drain does a non-Umbra mod have in an Umbra-polarized slot (match, mismatch, or neutral)? UNVERIFIED.
3. Riven drain at rank 8 (believed 10–18, user input) — no wiki source found.
4. One riven per weapon — no wiki source found; widely known but UNVERIFIED as sourced.
5. Can exalted weapons equip arcanes (all types or only primary exalted)? Only indirect evidence (Primary Merciless notes).
6. Exact semantics of a mod with several `CompatibilityTags` (all required vs any) — not documented.
7. Does a spliced trait's value use the riven's L for its +/− layout? Not stated.
8. Is riven Damage really negative-capable (overview tables, patch example) despite the "positive only" footnote?
9. Does Vigilante's enhancement ever turn a non-crit into a crit? Page wording says crits only.
10. Strain Infection total = 20% × cysts from the set table (max 160%)? Derived, not stated.
11. Sacrificial Sentient-damage values at set level 2 (x1.65 vs the notes' "+41.3%").
12. Is the Coda valence distribution uniform too (U42 note names only Kuva Lich and Sisters weapons)?
13. Minimum capacity formula (Mod vs Polarity page) — irrelevant for max-rank builds but contradictory.
14. Which "Increase Base Damage by +X" incarnon perks interact with GunCO like the Braton bug (only Braton is documented)?
