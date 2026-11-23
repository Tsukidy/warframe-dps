Status: draft

# Melee damage mechanics (game-truth side)

Researched 2026-11-23 from wiki.warframe.com wikitext and Lua data modules (cached under
`research/external/cache/wiki/` and `research/external/cache/wiki-modules/raw/`). Page revision
timestamps checked via the MediaWiki API on 2026-11-23: `Melee` 2026-11-17, `Melee Combo` 2026-10-28,
`Stance` 2026-10-02, `Module:Stances/data` 2026-10-02, `Module:Weapons/data/melee` 2026-10-22,
`Condition Overload` 2026-10-12, `Exalted Blade` 2026-11-12, `Finisher` 2026-09-23, `Stealth`
2026-06-19. The newest patch referenced on the `Melee` page is Update 44. No other site was needed;
where the wiki is silent the item is marked **UNVERIFIED**.

Related files that already cover parts of this topic (do not duplicate, cross-check against):
`wiki-additive-vs-multiplicative-edge-cases.md` §1.10 (CO per stance), §2 (combo-scaled crit/status),
§12 (melee projectiles and slams); `wiki-core-damage-formula.md`; `data-source-wiki-modules.md`
(Stances/data parser caveats).

Notation: `B` = weapon base damage (sum of the per-type values of the "Normal Attack" in
`Module:Weapons/data/melee`), `AS` = attack speed, `HAE` = heavy attack efficiency, `M(c)` = combo
multiplier for combo count `c`.

---

## 1. Attack speed and hits per second

### 1.1 Formula

`Attack Speed` page § Mechanics:

```
Modified Animation Length = Base Attack Animation Length / [ Base Weapon Attack Speed × (1 + Σ Attack Speed Bonuses) ]
```

- *Base Attack Animation Length* = the length of the stance combo at attack speed 1.0. The wiki stores
  it as `Duration` (seconds) per combo in `Module:Stances/data`; `Stance` page § Comparison: "Length is
  the base duration of a particular combo when melee attack speed is at 1.0".
- *Base Weapon Attack Speed* = the weapon's `FireRate` field on its "Normal Attack" in
  `Module:Weapons/data/melee` (e.g. Skana 0.833, Nikana Prime 1.08).
- Bonuses are additive with each other: Fury +30 %, Primed Fury +55 %, Gladiator Vice +30 %,
  Quickening +40 %, Berserker Fury +35 % per stack ×2 on melee kill (10 s), Arcane Strike +60 % (15 %
  chance on any hit, 18 s), Excalibur Swordsmanship +10 % (swords, dual swords, nikanas, rapiers,
  Exalted Blade), Spoiled Strike −20 %; abilities (Warcry, Speed, Penance, Redline, Reservoirs). Sources:
  `Module:Mods/data` descriptions; `Arcane Strike` § Notes ("additive to mods like Fury");
  `Excalibur/Abilities/Passive` ("additive to mods such as Pressure Point and Fury").
- `Attack Speed` page: "All base weapon Attack Speeds are in fractions of X / 60", so displayed values
  are rounded (0.833 = 50/60; 1.08 is probably 65/60 = 1.0833). **UNVERIFIED** which exact fraction
  each weapon uses; recommendation: snap `FireRate` to the nearest 1/60 and keep a test.

Hits per second for a looping stance combo with `N` hits and summed per-hit multiplier `ΣD`:

```
loop_time   = Duration / (FireRate × (1 + ΣAS))
hits_per_s  = N / loop_time
mult_per_s  = (ΣD / 100) / loop_time          (stance damage multiplier per second)
DPS_direct  ≈ ModdedHitDamage × mult_per_s     (single target, before crit/status/enemy)
```

The wiki's own average-DPS formula (`Stance` § Comparison, flagged `{{UpdateMe}}` because it omits
stance physical bonuses):
`Average DPS = Modded Melee Damage × Modded Attack Speed × (Total Damage Multiplier + Total Slash Proc Multiplier) / Base Combo Length`,
with `Total Slash Proc Multiplier = Σ(Dmg × Hits over forced-Bleed hits) × 0.35 × 6`
(`Module:Stances` `calcTotalSlashProcMulti`).

### 1.2 What attack speed does and does not change

- Speeds every stance combo, slide/aerial attacks and the **swing** of a heavy attack, **not** the
  heavy wind-up (`Attack Speed` § Mechanics; `Melee` § Wind Up: "Increasing melee attack speed does not
  reduce the wind-up time; rather, it reduces the interval between Heavy Attacks").
- Glaive / Wolf Sledge ranged charge (throw) attacks are **not** affected by attack speed, only by
  Heavy Attack Wind Up Speed (`Attack Speed` § Mechanics).
- Finisher animations: only attack-speed **mods/arcanes** count (not the weapon's base speed), capped at
  33 % faster at +50 % (`Finisher` intro list). For a DPS tool this only matters for a finisher-loop mode.

### 1.3 Worked example (hits per second)

Nikana Prime (`FireRate` 1.08), Blind Justice neutral combo "Guiding Light" (`Duration` 2.6 s; hits
300, 200, 200, 200, 200, 200, 200, 400 % → N = 8, ΣD = 1900 %):

| Attack speed mods | AS | loop_time | hits/s | multiplier/s |
|---|---|---|---|---|
| none | 1.08 | 2.6 / 1.08 = 2.4074 s | 3.323 | 7.892 |
| Primed Fury + Gladiator Vice + Arcane Strike (+145 %) | 1.08 × 2.45 = 2.646 | 0.9826 s | 8.141 | 19.336 |

Average stance multiplier per hit = 19.0 / 8 = 2.375 either way. (Computed in this session from
`Module:Stances/data` and `Module:Weapons/data/melee`.)

---

## 2. Stances and stance combos

### 2.1 Mechanics (`Stance` page; `Melee` § Stances)

- Four stance-specific combos per stance: Neutral (melee only), Forward (moving), Block (block/aim +
  melee, a.k.a. Neutral Tactical), Forward Block. Heavy, Slide, Aerial, Wall, Ground Finisher and Slam
  are shared by the weapon class and unchanged by the stance mod. Without a stance mod a weapon uses its
  class "stanceless" combos.
- Each hit carries a damage multiplier (100 % = weapon damage), may force status procs
  (Slash = "Bleed", Impact, Lifted, Knockdown, Ragdoll, Stagger, ...) and may add a physical bonus.
- Stance physical bonuses multiply **multiplicatively** with physical mods (`Stance` § Notes):
  `Physical Damage = Base Physical × (1 + Stance Physical Bonus) × (1 + Modded Physical Bonus) × Combo Damage Multiplier On Hit`.
  In the data these are `SlashMultiplier`/`ImpactMultiplier`/`PunctureMultiplier` per hit, e.g. 1.1 =
  +10 %, 2 = +100 %.
- Combo-embedded "Slam" hits are radial; `Module:Stances` `calcTotalDmgMulti` excludes them from the
  single-target multiplier ("a combo with a slam attack will only produce one damage instance on direct
  hit"). U35.5 patch notes (on `Melee` § Patch History): "Slam attacks during combos deal flat damage
  amounts and do not scale with Mods. We currently do not have plans to change this."
- Since U31 some weapons override part of the stance with unique combos: Sigma & Octantis and Cobra &
  Crane (aerial), Nepheri (neutral), Korumm and Verdilac (block), Argo & Vel (heavy) (`Stance` § Weapons
  With Unique Combos).
- Stance switching mid-combo is seamless since U26 (`Stance` patch history) → a DPS tool can treat each
  combo as an independent loop.

### 2.2 Machine-readable stance data

| Source | Combo data? | Notes |
|---|---|---|
| wiki `Module:Stances/data` | **Yes** | 135 entries: 63 PvE stance mods, 28 stanceless (class default) tables, 21 Conclave-only, 22 `UniqueToWeapon` (Argo & Vel, Arum Spinosa, Azothane, Caustacyst, Cobra & Crane, Edun, Jat Kusar, Keratinos, Korumm, Nepheri, Quassus, Sampotes, Sigma & Octantis, Syam, Tatsu, Telos Boltace, Tenet Agendus, Tenet Exec, Tenet Grigori, Verdilac, Vitrica, Zenistar). Per combo: `Name`, `Duration`, `Attacks[]`; per attack arrays aligned by hit: `Dmg` (% multiplier), `Hits` (repeat count), `Procs` (string or list), `Types` (`360`, `Ranged`, `Slam`, `Sweep`, `Thrust`), `SlashMultiplier`/`ImpactMultiplier`/`PunctureMultiplier`, `StatusMultiplier` (6 uses), `Shape` (`Heavy`), `Note`. Top-level `WeaponType`, `StanceName`, `StancelessStance`, `ConclaveOnly`, `UniqueToWeapon`. The module ends `return addSharedCombos(StanceData)`, which copies the class's Heavy/Slide/Aerial/Wall/Finisher/Slam into each stance (Ravenous Wraith excluded). |
| WFCD `Mods.json` | No | 78 stance records with name/polarity/compat only (`data-source-wfcd.md`). |
| DE Public Export | No | stances present, combos absent (`data-source-public-export-and-connector.md`). |

`Duration` coverage (counted this session): PvE stance mods 258/269 combos have it; missing only on
Gleaming Talon "Block", Blind Justice "Wall", and the shared Aerial/Heavy/Slide/Slam/Wall/Finisher of the
exalted stances Exalted Blade, Primal Fury, Serene Storm, Ravenous Wraith. Stanceless (class default)
tables: only 72/191 combos have `Duration`; Sword, Nikana, Polearm, Hammer, Dagger, Claws, Fist, Machete,
Rapier, Sparring, Staff, Tonfa, Warfan, Glaive, Dual Daggers have **no** neutral duration. Shared Heavy
combos: 6/44 have `Duration` (Dual Nikanas, Dual Swords, Edun, Hysteria, Sampotes, Syam). So the heavy
swing time and most stanceless loop times are **not** available from the wiki → see Open questions.

Data quirks to handle in the parser:
- `Blind Justice` Forward Block hit 2 has `Procs = { "", Knockdown, "" }` (unquoted identifier). In real
  Lua this evaluates to `nil`, so the Knockdown proc silently disappears; a hand-written parser must treat
  bare identifiers as strings or as missing consistently.
- `Crimson Dervish` Block is byte-identical to Neutral ("Twisting Flurry"); possibly a data error.
- `StatusMultiplier` (values 1.5, 2) is not used by the wiki renderer (`Module:Stances`); its meaning
  (status-chance multiplier for that hit?) is **UNVERIFIED**.
- Non-integer per-hit multipliers exist: 40, 50, 125, 150, 166, 180, 185, 215, 220, 225, 250, 450, 555 %.
- Inspiration stance durations differ slightly from the wiki (Blind Justice "Zatōs Creed" 3.0 vs 3.05,
  "Heeding Call" 2.5 vs 2.45; Zatōs Creed hit list 6 vs 7 hits) — for the inspiration-comparison team.

---

## 3. Melee combo counter

Source unless stated: `Melee Combo` page (rev 2026-10-28).

### 3.1 Gain

- Stance attacks add combo points equal to the hit's stance multiplier / 100 **per enemy struck**
  (300 % hit → 3 points). Only successful hits on enemies count; destructible objects do not. Exceptions
  exist (last hit of Swirling Tiger "Dancing Hunter", first hit of Homing Fang "Cutting Arches").
- `Module:Stances` `calcTotalComboGain` rounds each hit's gain to the nearest integer
  (`floor(D/100 + 0.5)`), so a 50 % hit gives 1 point and 250 % gives 3. That is the wiki's tabulation
  rule, **UNVERIFIED** against the game.
- Each blocked enemy attack (each projectile) +1. Slams: +1 per enemy struck (`Melee` § Slam Attack).
  Stealth/finisher attacks add "varying amounts". Rauta: 2 per pellet, up to 28 from 14 pellets.
- Heavy attacks do **not** add combo (`Melee` § Wind Up: "connecting with a Heavy Attack does not add to
  the combo counter").
- Additional Combo Count Chance (additive, starts at 0 %): Quickening +20 %, True Punishment +100 %
  (−50 % combo duration), Enduring Strike +20 % (lifted targets), Guardian Derision +30 % (while
  blocking), Relentless Combination +100 % on Slash-proc damage ticks, Exodia Triumph +50 % and Exodia
  Valor +200 % on lifted (Zaws). Behaviour above 100 % (guaranteed +1 plus chance of +2?) is
  **UNVERIFIED**.

### 3.2 Multiplier tiers

```
M(c) = clamp( floor(c / 20) + 1, 1, 12 )
```

2× at 20, +1× per 20 hits, 12× at 220 (`Melee Combo` § Combo Multiplier table). Below 20 the multiplier
is 1× (a heavy at 0 combo is allowed and deals the plain heavy multiplier; `Melee` § Wind Up).
Exceptions: **Venka Prime** cap 13× at 240; **Dex Nikana** 11 hits per tier, cap 11× at 110
(`M = clamp(floor(c/11)+1, 1, 11)`).

- **Normal attacks are not multiplied by the combo multiplier** (`Melee Combo` § Mechanics). It feeds
  heavy attacks, Blood Rush, Weeping Wounds, the Gladiator set, Secondary Outburst, Lifted duration,
  Iron Staff range (+0.5 m per tier) and a few weapon traits (Pennant, Ruvox, Tonkkatt) (`Melee Combo` §
  Combo Scaling Effects).
- "Multiplier Ability Damage" abilities scale at 1 : 0.25 (1.5×–4×) without consuming combo; Ability
  Combo Counter abilities (Blade Storm, Landslide, Shattered Lash, Whipclaw) have their own 2×–12×
  counter whose duration also scales with Ability Duration (marked "verification needed" on the page).
- The counter value itself appears uncapped (heavy-attack efficiency is defined as a percentage of
  "current combo"); whether the count stops at some value is **UNVERIFIED**.

### 3.3 Duration and decay

- Base 5 s (weapon field `ComboDur`; exceptions: Guandao Prime 6 s, Pulmonars 9 s, Vitrica 10 s, Xoris
  infinite; Tenet Livia, Tenet Grigori and several Incarnon "Standoff"/"Abiding Hold" evolutions pause
  the timer while holstered; Combo Killer resets a glaive's timer on secondary kill).
- Refreshed by any hit/block that **adds** to the count; also by a heavy attack even when it hits
  nothing (`Melee Combo` § Tips). Paused during heavy wind-up (`Melee` § Wind Up).
- Flat additions are additive: Body Count +12 s, Drifting Contact +10 s, Gladiator Rush +6 s, Swift
  Momentum aura +6 s, Combo Killer +5 s, Amalgam Furax Body Count +15 s, Melee Guidance aura (−6 s self,
  +12 s squad), Primary/Secondary Dexterity +7.5 s each (other file). 5 + 12 + 10 + 6 = 33 s
  (`Body Count`, `Drifting Contact` notes).
- Percentage penalties apply to the **base** value: Corrupt Charge −50 % "always subtracts exactly
  2.5 seconds" (`Corrupt Charge` § Notes); True Punishment −50 %. Proposed formula
  `D = ComboDur_base × (1 + Σpct) + Σflat`, min 0.1 s; "A zero or negative combo duration prevents
  increasing the combo counter". Ordering when both penalties and flat bonuses are present is
  **UNVERIFIED**.
- On expiry the count resets to Initial Combo. With Naramon **Power Spike** it instead loses
  20/15/10/5 points (rank 1→max) each time the timer runs out, the timer restarting each time
  (`Focus/Naramon` § Power Spike; `Melee Combo` § Combo Decay: 220 s / 44 ticks to empty from 220 at
  5 s duration).

### 3.4 Initial Combo

Minimum combo when idle or after a reset; heavy attacks spend it and it regenerates at **40 points/s**
(`Melee Combo` § Initial Combo). Sources: Corrupt Charge +30 (rank 3), Covert Lethality +16, Galvanized
Reflex +20 per melee kill for 20 s ×4 (+80), Ready Steel aura +24 (mission start), Melee Crescendo +6 per
finisher kill for the rest of the mission (unbounded; 37 kills → permanent 12×), innate Synoid Heliocor
20, Furax Wraith 20, Fragor Prime 30; Incarnon evolutions (Magistar +30, Ceramic Dagger +20 and +1 per
primary kill ×100, Destreza/Praedos/Ruvox/Thalys +20, Hate +30 with Dread and Despair, Sibear +15 ×4).

### 3.5 Worked example: combo over a 20 s fight

Same build as §1.3, single target, every hit connects, initial combo 0, hits spaced evenly over the
loop (an assumption — the wiki gives only total combo length). Each loop gives 19 points.

| | combo at 20 s | tier at 20 s | 12× reached at | time-averaged tier over 0–20 s |
|---|---|---|---|---|
| AS 1.08 (no mods), 66 hits | 157 | 8× | not reached (≈27.9 s) | 4.43 |
| AS 2.646, 162 hits | 385 | 12× | 11.42 s | 8.57 |

Times the tiers are first reached (AS 2.646): 2× 1.11 s, 3× 2.09, 4× 3.07, 5× 4.18, 6× 5.16, 7× 6.26,
8× 7.25, 9× 8.35, 10× 9.34, 11× 10.44, 12× 11.42 s. Consequence for Blood Rush: with True Steel (+120 %)
and Blood Rush on a 28 % base, crit chance at 12× is `0.28 × (1 + 1.2 + 4.4) = 184.8 %`, but the
hit-averaged crit chance over the first 20 s is only 101 % (AS 1.08) or 146.9 % (AS 2.646). A DPS tool
that assumes 12× from t = 0 overstates short fights; it should either let the user fix the tier or
simulate the ramp. (Simulated in this session.)

---

## 4. Heavy attacks

Source unless stated: `Melee` § Heavy Attack (rev 2026-11-17).

### 4.1 Damage

```
Heavy = B × (1 + DamageMods + HeavyOnlyMeleeDamage) × (1 + ElementalMods) × HeavyTypeMult × M(c)      (before crit)
Heavy Slash total (6 ticks) = [B × (1 + DamageMods) × (1 + Faction)] × HeavyTypeMult × M(c) × (1 + Faction) × 0.35 × 6
```

- `HeavyTypeMult` per class (first / second-or-slide heavy; typical wind-up):
  Assault Saw 6× (3×100 % + 3×100 % Bleed) / 6×, 1.0 s · Blade and Whip 4× / 12× (3×400 %), 0.4 s ·
  Claws 5× Bleed / 5× Bleed, 0.6 s · Dagger 5× (250 % Knockback+Bleed + 250 %) / 5×, 0.4 s · Dual
  Daggers 5× (2×250 % Bleed) / same, 0.5 s · Dual Nikanas 12× (3×200 % + 600 %) / 6× (100+200+300 %),
  1.0 s · Dual Swords 5× / 5×, 0.7 s · Fist 5× / 5×, 0.6 s · Glaive 2× / 3×, 1.2 s · Gunblade 5× / 5×,
  0.4 s · Hammer 6× / 6×, 1.2 s · Heavy Blade 6× / 6×, 1.1 s · Heavy Scythe 6× (3×200 %) / 12×
  (2×600 %), 1.0 s · Machete 6× (150 % Bleed + 3×150 %) / 5×, 0.7 s · Nikana 5× Bleed / 5× Bleed, 0.5 s ·
  Nunchaku 5× / 5×, 0.5 s · Polearm 6× / 6×, 0.9 s · Rapier 4.5× Bleed / same, 0.5 s · Scythe 6× Bleed /
  same, 1.0 s · Sparring 12× / 15× (3×300 % + 2×300 % Bleed), 0.5 s (Hirudo 6.5×, Korrudo 10.7× per an
  HTML comment) · Staff 5×, 0.5 s · Sword 5×, 0.6 s · Sword and Shield 5×, 0.7 s · Tonfa 5× (2×250 %
  Bleed), 0.7 s · Two-Handed Nikana 6× Bleed / 6× (2×300 % Bleed), 0.7 s · Warfan 5× (2×250 % Bleed) /
  5× Bleed, 0.5 s · Whip 4.5× Bleed, 0.4 s.
- Per weapon, `Module:Weapons/data/melee` has `HeavyAttack` (total, e.g. Skana 600 = 5 × 120, Fragor
  Prime 1620 = 6 × 270) and `WindUp`; prefer these over the class table, since many weapons deviate.
- The two heavies alternate; the slide heavy is usually the second (`Melee`; `Stance` § Combo Types).
- `HeavyOnlyMeleeDamage` ("Melee Damage on Heavy Attack"): Killing Blow +120 %, Melee Assimilation
  +150 % (after a shield break, 20 s), riven splice; additive with Pressure Point; heavy attacks and heavy
  slams only (`Melee` § Melee Damage on Heavy Attack).
- **Incarnon Genesis flat base damage is not multiplied by the Heavy Attack Multiplier but is multiplied
  by the combo multiplier** (`Melee` § Heavy Attack Damage).
- Heavies can also roll normal status besides forced procs.
- Crit chance: True Steel family (True Steel +120 %, Sacrificial Steel +220 %, Galvanized Steel
  +110 %, Amalgam Ripkas True Steel) and melee riven crit chance apply **twice** on heavy attacks
  (`Melee` § Interactions with Heavy Attacks; mod descriptions "x2 for Heavy Attacks"; Galvanized Steel
  U37.0.4 fix: heavy = +220 %, not +330 %). Melee Animosity: +42 % crit chance per normal-attack input
  (one stack per input regardless of enemies hit) for the next heavy, max +420 % after 10 inputs,
  consumed only if the heavy connects; additive with True Steel (`Melee Animosity` § Notes). Dreamer's
  Wrath +32 % crit damage on Tennokai heavies (additive with Organ Shatter).
- Glaives: heavy-attack effects apply to the explosion only, not to direct throw hits (`Melee` §
  Interactions). Non-hitscan projectiles from heavies (Destreza Incarnon, Syam) often do not count for
  heavy-attack bonuses like Killing Blow except while still in the wind-up.

Wiki examples (reproduced): Paracesis + Pressure Point + Fever Strike at 7×:
`222 × 2.2 × 1.9 × 6 × 7 = 38,974.32`. Reaper Prime + Pressure Point + Smite Grineer at 5× (Slash):
`200 × 2.2 × 1.3 × 6 × 5 × 1.3 × 0.35 × 6 = 46,846.8` (6 ticks of 7,807.8).

### 4.2 Combo consumption, HAE, Tennokai

- A heavy consumes the combo: `c_after = HAE × c_before`, `HAE = min(0.9, ΣHAE)` (`Melee Combo` § Heavy
  Attack Efficiency: 40 % at 160 → 64 retained, "capped at 90%"). Sources: Reflex Coil +60 %, Galvanized
  Reflex +50 %, Focus Energy/Focus Radon +40 %, riven +27.5–136 %, Kullervo passive +75 %, Bo Incarnon
  +50 % (+20 %), Destreza Incarnon +30 %, Furax Incarnon +90 % (heavy slams), Skana Incarnon +80 % with
  overshields, Corufell/Dual Keres +20 % with their frame, Praedos/Ruvox/Innodem conditional.
  Additivity of HAE sources is implied by the single 90 % cap, **UNVERIFIED** explicitly.
- If `c_after` < Initial Combo it regenerates at 40 points/s back to Initial Combo. Wiki: without HAE
  7× (120 points) takes 3 s, 12× (220) 5.5 s; at 50 % HAE 1.5 s / 2.75 s; at 90 % 0.3 s / 0.55 s.
- **Tennokai** (`Melee` § Tennokai): with any Tennokai mod, each qualifying melee hit has a 15 % chance
  to open a 2 s window; a heavy attack or heavy slam in the window has faster wind-up (not affected by
  other wind-up bonuses) and does **not** consume combo. Hits: one per input that strikes an enemy;
  multiple enemies or multi-strike hits do not count, except gunblade shots; Exalted Blade and Serene
  Storm waves count; heavy attacks do not count but a heavy slide does (`Discipline's Merit` notes).
  Mods: Mentor's Legacy (enable), Condition's Perfection +100 % status chance on Tennokai attacks
  (additive with Melee Prowess), Discipline's Merit (every 4 hits instead of random), Dreamer's Wrath
  (×1.5 chance → 22.5 %, +32 % crit damage), Master's Edge +60 % Tennokai damage (additive with Pressure
  Point), Opportunity's Reach (window 4.0 s, +3 m range), Truth's Flame (chained 4 s windows, +120 %
  damage after a Tennokai kill, additive; failure resets combo and burns the user). The Tennokai wind-up
  speed value is **UNVERIFIED**.

### 4.3 Wind-up and cycle time

- `WindUp` per weapon (s). Heavy Attack Wind Up Speed: Killing Blow +60 %, Amalgam Organ Shatter +60 %,
  Melee Elementalist +60 %, Swift Momentum aura +30 %, Kullervo +100 %, riven splice. "There is no cap
  ... diminishing returns" (`Melee` § Wind Up) suggests `windup = WindUp_base / (1 + ΣWindUpSpeed)`
  (**UNVERIFIED**; Glaive Prime 1.2 s with Killing Blow → 0.75 s under that formula). Heavy slams have no
  wind-up. Glaive wind-up went 0.6 → 1.2 s in U30.5.
- Cycle time of a heavy-only loop: `max(windup + swing / AS, regen_time)` where
  `regen_time = (Initial − HAE × c) / 40` if positive. The **swing duration** per class is not in the
  wiki data (only 6/44 shared heavy combos have `Duration`) → Open question.

### 4.4 Worked example (heavy attack)

Fragor Prime (B = 270, Hammer 6× → 1620, CC 40 %, CM 2.5, wind-up 1.2 s, innate Initial Combo 30). Mods:
Primed Pressure Point +165 %, Killing Blow +120 % heavy / +60 % wind-up, Organ Shatter +90 %, True Steel
+120 % (×2 = +240 % on heavy), Corrupt Charge (+30 initial), Galvanized Reflex 4 stacks (+80 initial,
+50 % HAE). Initial combo = 30 + 30 + 80 = 140 → 8×.

- Non-crit: `270 × 6 × (1 + 1.65 + 1.2) = 6,237` per combo tier → 8×: **49,896**; 12×: **74,844**.
- Crit: CC = `0.40 × (1 + 2.4) = 136 %`, CM = `2.5 × 1.9 = 4.75`, average crit multiplier
  `1 + 1.36 × 3.75 = 6.1` → 8×: 304,365.6; 12×: 456,548.4.
- Adding Blood Rush at 12×: if it uses the combo **before** consumption, CC = `0.40 × (1 + 2.4 + 4.4) =
  312 %` → 950,518.8 average; which combo value it reads is **UNVERIFIED** (same open item as
  `wiki-additive-vs-multiplicative-edge-cases.md` §2).
- Cycle: heavy at 140 retains 70; regen 70 / 40 = 1.75 s; wind-up 1.2 / 1.6 = 0.75 s (formula
  UNVERIFIED) + unknown swing time. So this build is regen-limited at ≈1.75 s per heavy unless the swing
  is longer than 1.0 s.

### 4.5 Heavy slams

3× weapon damage (`Module:Weapons/data/melee` "Heavy Slam Attack", Blast by default), receives the combo
multiplier and consumes combo, radial with falloff to 70 % at the edge, radius = class heavy-slam radius
scaled 50 %…150 % by drop height (≥15 m for max), applies Lifted for 1 s + 1 s per combo tier (not
status-duration scalable), no wind-up (`Melee` § Heavy Slams).

---

## 5. Combo-scaled bonuses (Blood Rush, Weeping Wounds, Gladiator)

Detailed in `wiki-additive-vs-multiplicative-edge-cases.md` §2; confirmed here from the same pages:

```
CC = CC_base × [1 + ΣCC_mods + 0.40 × BR × (M − 1) + 0.10 × n_gladiator × (M − 1)] + ΣCC_flat
SC = SC_base × [1 + ΣSC_mods + 0.40 × (M − 1)]          (Weeping Wounds)
```

- Blood Rush +40 % per tier above 1 (rank 10), relative crit chance bucket since U26 (was 165 %
  multiplicative), U30.5 60 → 40 %. Rank table: 3.6/7.3/10.9/14.6/18.2/21.8/25.5/29.1/32.8/36.4/40 %.
  Example `20 % × (1 + 1.2 + 0.4 × 3) = 68 %` at 4× (`Blood Rush` § Calculation).
- Weeping Wounds +40 % per tier above 1 (rank 5: 6.7/13.3/20/26.7/33.3/40 %); example
  `20 % × (1 + 0.6 + 0.4 × 3) = 56 %`.
- Gladiator set: +10 % crit chance per equipped set mod (3 Warframe + 3 melee mods: Finesse, Resolve,
  Aegis, Vice +30 % AS, Might +60 % CD, Rush +6 s combo duration) per tier above 1 (6 mods at 12× →
  +660 %); stacks with Blood Rush "+100 % per combo multiplier" at full set (`Gladiator Set`).
- Thrown glaive attacks are affected by the combo counter and so by Blood Rush (`Blood Rush` § Notes).
- Blood Rush / Weeping Wounds apply to Garuda Talons and exalted melee (fixed U29.5).

---

## 6. Condition Overload on melee

`Condition Overload` (mod) page: +80 % melee damage per unique status type on the target (rank 5;
120 % → 80 % in U30.5; exponential before U26).

```
Total = B × [1 + DamageMods + 0.8 × n] × (1 + ElementalMods)
```

- Does **not** apply to slams, heavy slams, or radial explosions (glaive heavy-attack explosion).
- Counts toward base damage for procs (Slash etc.); radial procs use the initial target's n.
- The status a hit applies does **not** count for that same hit (or its proc); gunblade pellets hit
  sequentially, so earlier pellets of a shot can raise n for later ones.
- Counts all damage-type procs plus Void, Tau, Microwave, **Lifted** from heavies and **Knockdown** (no
  visible icon). Lifted and Knockdown cancel each other → maximum 16 types.
- Some stance inputs multiply CO by less or more than expected (`Condition Overload (Mechanic)` §
  Stances: e.g. Gnashing Payara neutral 36 %, Mountain's Edge heavy 233 %, Exalted Blade 122–130 %,
  gunblade stances 100 %); full table summarised in `wiki-additive-vs-multiplicative-edge-cases.md`
  §1.10 and should be imported as data. U31.1.3 fixed CO "not accounting for Melee weapons that deal
  different amounts of damage with different swings via combos".

---

## 7. Melee arcanes (all values at max rank)

Sources: `Module:Arcane/data` descriptions; individual pages (`Melee Doughty`, `Melee Careen`,
`Melee Retaliation`, `Melee Assimilation`, `Melee Afflictions`, `Melee Influence`, `Melee Duplicate`,
`Melee Exposure`, `Melee Crescendo`, `Melee Animosity`, `Arcane Fury`, `Arcane Strike`,
`Arcane Blade Charger`).

| Arcane (introduced) | Effect | Bucket / DPS-relevant rule |
|---|---|---|
| Melee Animosity (U35) | +42 % CC per normal input to next heavy, max +420 % | relative CC, heavy only; see §4.1 |
| Melee Assimilation (U43.0) | after shield break: +150 % melee damage on heavy for 20 s; heavy kills restore 30 % shields | additive with Pressure Point, heavy/heavy-slam only; conditional |
| Melee Careen (U41) | ×2.5 melee damage vs fully frozen (10 Cold stacks) enemies; roll applies 10 Cold stacks in 5.5 m (2 s CD) | separate multiplier **UNVERIFIED** bucket (description "x2.50"); frozen enemies also take +1.0× critical damage from any source (`Melee Careen` § Notes) |
| Melee Crescendo (U35) | +6 Initial Combo per finisher kill for the mission | only for finishers with the weapon it is on (U38.5.3), Parazon finishers count for all; channeled exalted weapons only gain stacks while active |
| Melee Doughty (U38) | +1.0× crit damage per 10 % "Puncture status chance" | `bonus = round1( PunctureShare × SC% × 0.1 × 1.0 )`, cap +50×; SC includes Weeping Wounds, so it scales with combo. Example: Destreza Prime + Auger Strike + Drifting Contact + Weeping Wounds at 12×: `119 × 2.2 / 312.8 × 20 × (1 + 0.4 + 4.4) × 0.1 = 9.71 → +9.7×`. Applies only to the Puncture-bearing part of an attack (U38.6). Adding to CM flat vs relative: **UNVERIFIED** |
| Melee Duplicate (U35) | on a base (yellow, tier 1) crit: 100 % chance to strike again | separate instance, own crit and status rolls (can be non-crit/yellow/orange), forced weapon procs repeat, faction once (`Extra Hit`). Optimal CC = `(3·CM − 4) / (2·CM − 2)`; this formula only holds if orange/red crits do **not** trigger it (derived below), so treat as tier-1 only |
| Melee Exposure (U35) | on ability cast: +60 % Corrosive, stacks to +240 %, 25 s shared timer | elemental bucket (does not need combining); works on exalted/pseudo-exalted weapons |
| Melee Influence (U35) | on melee Electricity proc: 20 % chance for a 18 s buff (no refresh) that spreads elemental procs (not physical/Void/Tau/CC) in 20 m | spread damage = that element's quantized damage of the original hit incl. CO (target's n) and crit; faction applied twice on spread hits, thrice on spread DoTs; one-shot kills cannot trigger |
| Melee Afflictions (U36) | +6 stacks of every existing status when an enemy is knocked down / ragdolled / lifted by melee | new stacks use averaged existing proc damage; formula in `Melee Afflictions` § Damage Calculation; trigger matrix depends on enemy state |
| Melee Retaliation (U35) | +30 % melee damage per 200 current shields, max +420 % (2,800 shields; overshields count half) | additive with Pressure Point |
| Melee Fortification (U35) | on melee kill +210 armor 10 s | no damage |
| Melee Vortex (U35) | Magnetic-proc kill: 45 % chance to pull in 18 m | no damage |
| Arcane Fury | on crit: 60 % chance, +180 % melee damage, 18 s | additive with Pressure Point |
| Arcane Strike | on hit: 15 % chance, +60 % attack speed, 18 s | additive with Fury |
| Arcane Blade Charger | on primary kill: 30 % chance, +300 % melee damage, 12 s | additive with damage mods (`120 × (1 + 1.2 + 2.0)` example) |
| Exodia arcanes (Zaw only) | Brave (energy on heavy kill), Contagion (aim-glide projectile), Epidemic (slam shockwave), Force (on status: 50 % chance 200 % weapon damage in 6 m), Hunt, Might, Triumph (+50 % combo count chance), Valor (+200 % on lifted) | Contagion has its own two-bucket rules (other file §12) |

Derivation of the Melee Duplicate optimum (this session): with `C` ∈ [1, 2] total crit chance,
P(tier 1) = 2 − C. Average multiplier per hit `A = 1 + C(CM − 1)`; if only tier-1 crits duplicate and the
copy is an independent hit, total = `A × (1 + (2 − C)) = (1 + C(CM − 1))(3 − C)`; `d/dC = 0` gives
`C = (3CM − 4)/(2CM − 2)`, exactly the wiki formula. For C < 1 the total `(1 + C(CM−1))(1 + C)` grows
monotonically. So the wiki's optimum implies tier-1-only triggering.

---

## 8. Stealth and finisher damage

`Stealth` § Stealth Damage Bonus; `Finisher` page.

- **Stealth Damage Bonus** on normal melee attacks against unalerted, blinded, Silenced, or
  finisher-stunned enemies, or while the player is invisible:
  `SDB = 1 + 0.2 × Rank` (rank 30 → +700 % → ×8; capped at rank 30); rank-40 weapons
  `SDB = 1 + 0.15 × Rank` (→ +800 % → ×9) (`Finisher` § Stealth Attack). Normal hit:
  `Damage × (1 + SDB)`. With a crit, the bonuses **add**:
  `Damage × (1 + SDB + CritTier × (CM − 1))` (`Stealth` § Stealth Damage Bonus). 3 s per-enemy cooldown
  after being revealed.
- **Stealth / back / front / universal finishers**:
  `Finisher Damage = WeaponDamage × ClassFinisherMult × (1 + SDB)`; damage converted to Finisher (true)
  damage, ignoring armor. Crits: `Finisher` page writes `(1 + SDB + Critical Multiplier)` whereas
  `Stealth` writes `CritTier × (CM − 1)` → contradiction, **UNVERIFIED**. "Elemental and Physical damage
  mods only apply half of their bonus before being converted" (`Finisher` § Stealth Attack).
- Class multipliers (ground / stealth / back / front / universal), from `Finisher` § All Finishers
  Multiplier Comparison: Sword 10× / 16× (8+8) / 8× / 8× / 8×; Hammer 12× / 12× / 24× / 12× / 12×;
  Heavy Blade 12× / 12× / 8× / 2× / 2×; Nikana 10× / 8× / 8× / 16× / 8×; Dagger 10× / 6× / 6× / 6× /
  6×; Polearm 12× / 6× / 6× / 6× / 6×; Exalted Blade 12× / 16× / 8× / 8× / 8× (ability page instead
  says ground 400 %, prompted 3,200 %); Valkyr Talons 54× / 4× / 16× / 16× / 2.4×; full table in the
  cached page (`Heavy Scythe` flagged `{{UpdateMe}}`). Rapier front/back finishers proc Slash at 4×
  instead of 8×. Example: Skana Prime (210) rank 30 stealth finisher `210 × 16 × 8 = 26,880` (unmodded,
  non-crit).
- Finisher Damage mods: Finishing Touch +60 %, Covert Lethality +100 % (daggers; +16 initial combo),
  Hunter's Bonesaw (ground), Radiant Finish, Savage Silence. Bucket **UNVERIFIED** here (other file
  lists a "finisher bucket").
- Ground finishers ignore armor (U36) and deal Finisher damage which does not proc status.
- Known bug: weapons with Valence bonuses deal half the expected finisher damage (`Finisher` § Bugs).

---

## 9. Slam, slide, wall, aerial attacks — should a DPS tool model them?

- **Slam** (U35.5 / U39): 2× weapon damage, single element (Impact by default, weapon-specific
  overrides, e.g. Xoris Electricity), purely radial, falloff 100 % → 50 % at edge, radius per class 5–9 m
  scaled by drop height (min 2.5 m, max 150 % at ≥15 m), own status chance (often 10 %), forced Impact
  on most weapons. Heavy slam see §4.5. **Slam Damage** is a separate multiplier, multiplicative with
  Pressure Point: Seismic Wave +200 %, Nira's set +150 % per mod (fixed U39 from a double-application
  bug), riven splice; Arca Titron's Slam Capacitor adds base damage instead (`Melee` § Slam Damage).
  CO does not apply. Per-weapon values: `Module:Weapons/data/melee` attacks "Slam Attack" /
  "Heavy Slam Attack" with `Falloff`.
- **Slide**: `SlideAttack` field per weapon (often 2 × B, but Skana 75 vs B = 120 and Skana Prime 210 =
  1 × B are inconsistent → treat the field with caution) and stance `Slide` combos (28/41 have
  `Duration`). Pressure Point and elemental mods apply; "Critical Chance for Slide Attack" mods (Maiming
  Strike family, riven) apply only here.
- **Wall attack**: multiplier per class 1.67×–3× (`Melee` § Wall Attack table); not shown in the arsenal
  since U26.
- **Aerial**: stance-class `Aerial` combos (mostly no `Duration`).
- Recommendation: model slams and heavy slams as an optional "slam build" mode (they are a real build
  archetype with Seismic Wave/Nira's set) using the per-weapon slam attack and falloff data; skip wall
  and aerial attacks; model slide attacks only if a slide `Duration` exists. Ground finishers and
  stealth only as an explicit "finisher" mode, since they depend on crowd control.

---

## 10. Follow-through and punch-through

- `Melee` § Follow Through: n-th enemy in one strike takes `FT^(n−1)`; total over n enemies
  `(1 − FT^n)/(1 − FT)`, limit `1/(1 − FT)`. Example Skana FT 0.6, 3 enemies: `1 + 0.6 + 0.36 = 1.96`.
  Class defaults 0.4 (Hammer, Heavy Scythe) … 1.0 (Assault Saw, Exalted weapons); weapon field
  `FollowThrough`. Exceptions: Volnus native 0.6 reverts to 0.4 with a stance mod; Ironbride 0.7; Magistar
  Incarnon 0.8 with a channeled ability; Nami Solo Incarnon 1.0 in melee-only mode; Dorrclave 1.0 for 10
  attacks after 10 kills. Does not apply to slams or to projectile/AoE attacks (glaive throws and
  explosions). Whether dual-wielded weapons have per-hand follow-through is flagged `{{UpdateMe}}`.
- Combo gain is per enemy struck, so multi-target hits build combo faster (combo is not scaled by FT).
- Punch-through: normal melee strikes can benefit from it; sources are mostly glaive mods (Power Throw
  +2 m) and charged glaive throws; Exalted Blade waves punch through everything to max range
  (`Punch Through`; `Exalted Blade`).

---

## 11. Glaives

`Glaive (Weapon Type)`, `Glaive`, `Power Throw`, `Module:Weapons/data/melee`.

- Attacks per glaive in the data: Normal Attack, Throw, Throw Bounce Explosion, Throw Recall Explosion,
  Charged Throw, Charged Throw Bounce Explosion, Charged Throw Recall Explosion, Slam, Heavy Slam (13
  thrown weapons). Each has its **own** base damage, crit, status, forced procs and falloff (e.g. Glaive
  Prime: throw 180 IPS + forced Impact/Slash; recall explosion 592 Blast no falloff; charged recall
  explosion 1,184 Blast; bounce explosions 40 % falloff). `HeavyAttack` is 0 for glaives.
- The heavy-attack input detonates the glaive in flight; the explosion "is counted as a Heavy Attack and
  consumes the combo meter. It benefits from heavy attack mods and the combo multiplier" (`Glaive` §
  Notes). Explosion damage has no falloff since U29.5. Volatile Rebound / Volatile Quick Return explosions
  do **not** consume combo; Killing Blow does not raise them.
- Quick throws deal ≈50 % of charged throws (U30.5). Throw speed is governed by Heavy Attack Wind Up
  Speed (1.2 s base since U30.5), not attack speed.
- CO applies to the thrown glaive (U28.0.6) but **not** to the radial explosion.
- Power Throw: +2 m punch-through, +100 % throw damage per consecutive (buffered) throw up to 3 stacks;
  a "weapon damage" multiplier **multiplicative** with Pressure Point and additive with universal weapon
  bonuses (other file §12 for the two-bucket model).
- Heavy-attack crit doubling applies to standing-throw explosions; bug: airborne-throw explosions do not
  get the ×2 True Steel bonus, while Volatile-mod explosions always do (`Glaive (Weapon Type)` § Bugs).
- Melee Animosity: ground throws benefit but do not consume stacks (bug).
- The class heavy multiplier (2×/3×, `Melee` table) conflicts with `Module:Stances/data` Glaive heavy
  "Falling Star" (3 × 500 % ranged) → use the per-weapon explosion attacks, not either table.

## 12. Gunblades, whips, and other special classes

- **Gunblades** (`Gunblade`; data): "Ranged Attack" hitscan pellets (Redeemer Prime 80 Blast × 10
  multishot, falloff 10–30 m to 6.25 %, punch-through 1) alongside the melee "Normal Attack". Stance hits
  typed `Ranged` fire the shot. Shots use no ammo. Each shot counts as a Tennokai hit (multishot does
  not); pellets hit sequentially for CO. `HeavyAttack` 3,200 for Redeemer Prime = 4 × 800 — conflicts with
  the class table's 5× and with Stances/data "Full Bore" (2 × 400 % ranged + 100 % slam, then
  400 % + 100 %). Whether stance `Dmg` multiplies the shot damage, and how mods split between melee and
  shot, is **UNVERIFIED**. U28.0.6: gunblade heavies consume combo as intended.
- **Whips**: no class-specific damage mechanic on the wiki beyond class tables (heavy 4.5× Bleed, wind-up
  0.4 s, FT 0.5, block 45°); long range. Blade and Whip heavy 4× / 12×.
- **Heavy Scythe**, **Dual Nikanas**, **Sparring**: large second heavies (12×–15×). Sparring heavy in
  `Module:Stances/data` (500 % + 400 % lifted, then 4 × 100 %) contradicts the `Melee` table (12× /
  15×).
- Thrown-weapon hammers/polearms (Wolf Sledge, Sigma & Octantis shield throw) behave glaive-like on heavy.
- Weapons with heavy-attack projectiles/waves (Syam, Destreza Incarnon, Arum Spinosa, Quassus, Edun,
  Stropha, Zenistar, Tenet Agendus, Argo & Vel) have their own projectile entries and odd CO ratios
  (other file §1.7).

## 13. Exalted melee weapons

`Exalted Weapon`, `Exalted Blade`, `Hysteria`, `Primal Fury`, `Garuda Talons`, `Valkyr/Abilities/Passive`,
`Excalibur/Abilities/Passive`.

- Exalted melee: Exalted Blade (Excalibur), Valkyr Talons (Hysteria), Iron Staff (Primal Fury), Desert
  Wind (Serene Storm), Shadow Claws (Sevagoth), Diwata (Titania, no heavies). Pseudo-exalted (no heavies,
  no blocking): Shadow Clones (Blade Storm), Landslide Fists, Shattered Lash, Whipclaw; Slash Dash uses
  Exalted Blade's mods. Garuda Talons are a normal weapon with exalted-like traits (do **not** scale with
  Ability Strength).
- **Base damage scales with Ability Strength**; weapon rank only affects capacity. Exalted Blade normal
  250 at max ability rank (15 % Impact / 15 % Puncture / 70 % Slash, CC 15 %, CM 2.0, SC 15 %); Valkyr
  Talons 250 (25 % Puncture / 75 % Slash, CC 50 %, CM 2.0, SC 10 %; slide/wall 750, aerial 500, slam 300
  are also Strength-scaled); Iron Staff 300 (68 % Impact / 32 % Slash, CC 25 %, CM 2.0, SC 30 %).
  Steel Charge (aura, additive with Pressure Point) vs Growing Power (strength, multiplicative with CO) —
  `Hysteria` § Tips.
- They use their own combo counter (Body Count etc. on the exalted weapon itself; `Body Count` § Notes);
  Melee Crescendo stacks only while a channeled exalted is active.
- **Exalted Blade waves**: normal/slide/aerial attacks emit a wave with the same base damage as the hit;
  waves are separate damage instances, lose 40 % damage per second of flight (linear falloff 100 % → 0 %
  over 70 m), lose 20 % per enemy hit (normal waves only); heavy attacks emit 2 crossed 10 m waves that
  are weapon projectiles (hit weak points), do not lose damage per enemy, and do **not** scale with the
  combo counter. Waves add combo and trigger Tennokai. Slam radial damage has no crit and ignores combo.
  Swordsmanship passive +10 % damage and attack speed (additive). Exalted Blade stance CO ratios
  122–130 %; Exalted Umbra Blade light waves Adding (250 → 500 CO base) while heavy waves Multiplying (other
  file §1.7).
- Valkyr Rage passive: +3 % melee damage per hit, +12 % per kill, +27 % per finisher, max +300 %,
  additive with Pressure Point and CO; decays after 5 s idle (`Valkyr/Abilities/Passive`). Hysteria
  slide does 6 hits. The first Hysteria heavy is a slam (Seismic Wave applies).

---

## 14. Recommendations for the engine (non-binding)

1. Normal attacks: per selected stance combo, use `Duration`, per-hit `Dmg`, `Hits`, `Procs`,
   physical multipliers from `Module:Stances/data`; hits/s per §1.1; skip `Slam`-typed combo hits for
   single target (they are flat and radial).
2. Combo: either a user-fixed tier or a time simulation (§3.5) with initial combo, duration and
   Power Spike decay; default the DPS window to the inspiration's 20 s for comparability.
3. Heavy: separate attack mode using per-weapon `HeavyAttack` and `WindUp`, crit doubling for the steel
   family/rivens, heavy-only damage bonus, Incarnon-flat exception, HAE and 40/s regeneration, Tennokai
   as an optional "no consumption" flag.
4. Glaives and gunblades: model thrown/explosion/ranged attacks as their own attacks with their own base
   damage from the weapon module.
5. Slams, finishers and stealth: optional modes, not the default DPS.
6. Store `StatusMultiplier`, `Types` and `Note` from the stance data even if unused, so later findings
   can be applied without re-parsing.

---

## Edge cases

- Combo multiplier does **not** affect normal attacks; only heavies, heavy slams, glaive explosions,
  and combo-scaled mods (Blood Rush, Weeping Wounds, Gladiator, Secondary Outburst, Melee Doughty via
  Weeping Wounds).
- Venka Prime 13× cap at 240; Dex Nikana 11 hits per tier and 11× cap.
- Heavy attacks add no combo; slams add 1 per enemy; blocking adds 1 per blocked projectile.
- Incarnon flat base damage: not ×HeavyTypeMult but ×combo on heavies.
- True Steel family and riven crit chance ×2 on heavies; Galvanized Steel heavy = +220 % (U37.0.4 fix);
  glaive airborne-throw explosions miss the ×2, Volatile explosions always get it (bugs).
- CO does not apply to the hit that inflicts the new status; gunblade pellets can buff later pellets.
- CO does not apply to slams, heavy slams or radial explosions; Lifted (from heavies) and Knockdown count
  as statuses; Lifted and Knockdown cancel → max 16.
- CO stance ratios ≠ 100 % on many stance inputs (Gnashing Payara 36 %, Mountain's Edge heavy 233 %,
  Exalted Blade 122–130 %).
- Combo-embedded slam hits deal flat damage that does not scale with mods (U35.5).
- Stance physical bonuses are multiplicative with physical mods (not additive).
- Slam Damage (Seismic Wave, Nira's) is multiplicative with Pressure Point; Arca Titron's Slam Capacitor
  is base damage instead.
- Exalted Blade heavy waves ignore combo; its slam radial damage cannot crit.
- Exalted weapons scale with Ability Strength and keep their own combo counter; Garuda Talons do not
  scale with Strength.
- Stealth bonus and crit multiplier add (not multiply); finisher crit formula differs between pages.
- Finisher conversion halves elemental and physical mod bonuses.
- Weapons with Valence bonuses deal half finisher damage (bug).
- Melee Influence applies faction twice on spread hits and thrice on spread DoTs; one-shot kills don't
  trigger it.
- Melee Doughty reads modded status chance (incl. Weeping Wounds), rounds to 0.1, caps at +50×, only on
  the Puncture-bearing part.
- Melee Duplicate triggers on tier-1 crits only (implied by the wiki optimum), copies roll independently,
  faction applies once; it duplicates Toxic Lash / Xata's Whisper extra hits instead of the melee hit
  (bug).
- Glaive explosions consume combo but Volatile mod explosions don't; Killing Blow doesn't affect
  Volatile explosions or direct throws; throws are wind-up-speed-scaled, not attack-speed-scaled.
- Power Throw / Arcane Arachne etc. are "weapon damage" multiplicative with "melee damage" on glaive
  throws/explosions.
- Corrupt Charge and True Punishment penalties subtract from the base combo duration only.
- Volnus follow-through changes with a stance mod; Dorrclave/Magistar/Nami Solo conditional FT.
- Base attack speeds are X/60 fractions; displayed values are rounded.
- Finisher animation speed ignores the weapon's own attack speed and caps at 33 % faster.
- Tennokai: random 15 % per qualifying input (22.5 % with Dreamer's Wrath), deterministic every 4 hits
  with Discipline's Merit; gunblade shots each count.
- `Module:Stances/data` bare identifier `Knockdown` (Blind Justice) evaluates to nil in Lua.

## Open questions

1. Heavy-attack swing (recovery) time per class/weapon is not in the wiki data (6/44 heavy combos have
   `Duration`); needed for heavy DPS. Candidate external source: the u/GoldPhos "true attack speed"
   dataset linked from `Attack Speed` (not fetched) or in-game frame timing.
2. Neutral loop `Duration` missing for 15 stanceless classes (Sword, Nikana, Polearm, Hammer, ...).
3. Wind-up formula `base / (1 + ΣWindUpSpeed)` and the Tennokai wind-up bonus value: UNVERIFIED.
4. Does Blood Rush/Gladiator/Weeping Wounds use the combo before or after a heavy consumes it?
5. Additional Combo Count Chance above 100 %; rounding of fractional combo gain (wiki module rounds
   half-up per hit); is the raw combo count capped?
6. Combo duration formula order when % penalties and flat bonuses are combined.
7. Finisher crit formula: `(1 + SDB + CM)` (`Finisher`) vs `(1 + SDB + tier·(CM−1))` (`Stealth`); bucket
   of Finisher Damage mods; Exalted Blade finisher 400 %/3,200 % (ability page) vs table 12×/16×.
8. Heavy multiplier conflicts: Gunblade 5× (table) vs 4× per shot (Redeemer Prime 3,200) vs Stances/data
   Full Bore; Sparring 12×/15× vs Stances/data; Glaive 2×/3× vs Stances/data 500 %.
9. Gunblade stance `Ranged` hits: does the `Dmg` % multiply the shot damage, and do melee mods (Pressure
   Point) apply to shots?
10. Melee Careen ×2.5 and Melee Doughty bonus buckets (separate multiplier? flat CM add?).
11. Meaning of `StatusMultiplier` in stance data.
12. `SlideAttack` field inconsistencies (Skana 75 vs 120 base).
13. Exact X/60 fraction for each weapon's base attack speed.
