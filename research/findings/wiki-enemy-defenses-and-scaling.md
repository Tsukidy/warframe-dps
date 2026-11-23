Status: draft

# Enemy defenses and level scaling (game-truth side)

Author: research agent (wiki side). Date: 2026-11-23. Game state: Update 40-44 era (wiki pages edited up to 2026-11).

This file covers what the target does to incoming damage: health, shield, armor and overguard layers, faction damage-type modifiers, armor reduction and stripping, shield and overguard rules, level scaling, Steel Path and Eximus modifiers, and damage reduction or attenuation on special enemies. Weapon-side maths (quantization, crits, status damage) belongs to other findings files and is only mentioned where it touches defenses.

## 0. Sources and cache

All wiki pages were fetched with `?action=raw` and cached under `research/external/cache/`:

| Source | Cache path | Last wiki edit (API `rvprop=timestamp`) |
|---|---|---|
| Enemy Level Scaling | `cache/wiki/Enemy_Level_Scaling.wiki` | 2026-07-29 |
| Armor | `cache/wiki/Armor.wiki` | 2026-09-28 |
| Shield | `cache/wiki/Shield.wiki` | 2026-08-23 |
| Overguard | `cache/wiki/Overguard.wiki` | 2026-10-01 |
| The Steel Path | `cache/wiki/The_Steel_Path.wiki` | 2026-10-14 |
| Damage Reduction (includes the Update 40 patch notes on damage attenuation) | `cache/wiki/Damage_Reduction.wiki` | 2026-09-14 |
| Damage/Overview Table (faction modifiers) | `cache/wiki/Damage__Overview_Table.wiki` | 2026-10-01 |
| Damage/Heat Damage, Damage/Corrosive Damage | `cache/wiki/Damage__Heat_Damage.wiki`, `Damage__Corrosive_Damage.wiki` | 2026-11-14, 2026-10-26 |
| Eximus | `cache/wiki/Eximus.wiki` | 2026-11-16 |
| Per-faction pages Damage/<Faction> | `cache/wiki/Damage__*.wiki` | various |
| Enemy pages: Archon, Eidolon_Teralyst, Juggernaut, Lephantis, Hemocyte, Orphix, Treasurer, Condrix, Guardsman, Sentient, Shadow_Stalker, Acolytes, Demolisher, Necramech, Bursa, Deimos_Jugulus, Amalgam, Corrupted_Heavy_Gunner | `cache/wiki/<Name>.wiki` | Archon 2026-07-20, Treasurer 2025-11-01, Orphix 2026-08-13 |
| Module:Enemies/infobox (per-faction scaling coefficients used by the wiki's level slider) | `cache/wiki-modules/raw/Enemies_infobox.lua` | 2026-10-08 |
| MediaWiki:Gadget-enemyinfoboxslider.js (the wiki's own implementation of scaling, Steel Path, Eximus) | `cache/wiki-gadgets/enemyinfoboxslider.js` | 2026-10-07 |
| Module:DamageTypes/data, Module:Enemies/data/* (base stats) | `cache/wiki-modules/raw/DamageTypes_data.lua`, `Enemies_data_*.lua` | (cached earlier by another agent) |

Reddit (old.reddit.com) is blocked by the proxy, so the Reddit post the Archon page cites for attenuation values could not be read. The forums were not needed.

Calculations in this file were run with `python3` and numpy float32. The script is in the scratchpad and is not part of the repository; every number is reproducible from the formulas below.

---

## 1. Defense layers and the order damage meets them

Enemies can have up to four hit-point layers: Overguard, Shield, Health, and Armor. Armor is a damage reduction on Health, not a pool. (Source: Damage, U36 notes "Instead of having 13 Health Types, we have simplified them into 4: Health, Armor, Shields, Overguard".)

Order (sources: Overguard > Mechanics, Shield > Effects, Damage/Calculation > Armored Enemies):

1. **Overguard** absorbs damage first and protects shields and health. It is neutral to every damage type except Void (x1.5). Armor, pure damage reduction (e.g. Splinter Storm), and damage-type resistance effects do not apply to it. Damage Vulnerability does apply (Overguard > Mechanics). Toxin does **not** bypass Overguard (Damage/Toxin Damage, intro).
2. **Shield** comes next. Armor does not reduce shield damage. Toxin bypasses shields (with exceptions, section 5.1). Faction damage-type modifiers apply to shields (Damage > intro: "regardless of the presence of armor or shields").
3. **Health** comes last, reduced by armor if the enemy has more than 0 armor.

Viral status multiplies damage to health only (not shields, not overguard) (Damage/Viral Damage). Magnetic status multiplies damage to shields and overguard (Damage/Magnetic Damage, Overguard > Mechanics).

Bypass rules (Module:DamageTypes/data `Bypass` fields; Damage/Overview Table):
- Armor is bypassed by True, Finisher and Cinematic damage. Slash Bleed ticks are Cinematic, so they ignore armor (Damage/Calculation line 691: "Slash DoT deals Cinematic damage, ignoring armor").
- Toxin bypasses Shield (enemy and player) but not Overguard and not Armor (Damage/Toxin Damage, Patch History: "Toxin bypasses Shields (but not Armor) where as Slash Status bypasses Armor but not Shields").
- True damage bypasses Armor and Shield (`Bypass = { "Armor", "Shield" }` on True in Module:DamageTypes/data). True damage has no faction modifiers (Damage, line 393). UNVERIFIED: whether True damage bypasses Overguard; the module only lists Armor and Shield.

---

## 2. Faction damage-type modifiers (post-Update 36)

Since Update 36 (2024-06-18), resistances and vulnerabilities belong to the enemy's **faction**, not to its health type. There are only two magnitudes:

- Vulnerable: **x1.5** incoming damage of that type.
- Resistant: **x0.5** incoming damage of that type.

Source: Damage/Overview Table, header: "As of Update 36, Vulnerable + = x1.5 and Resistant - = x0.5 incoming damage multiplier." The table was parsed cell by cell and matches the `Positives`/`Negatives` lists in Module:DamageTypes/data `Types`.

| Faction (health class) | Vulnerable x1.5 | Resistant x0.5 | Source (patch) |
|---|---|---|---|
| Grineer | Impact, Corrosive | none | U36 |
| Kuva Grineer | Impact, Corrosive | Heat | U36 |
| Corpus | Puncture, Magnetic | none | U36 |
| Corpus Amalgam | Electricity, Magnetic | Blast | U36 |
| Infested | Slash, Heat | none | U36 |
| Infested Deimos | Blast, Gas | Viral | U36 |
| Orokin (Corrupted, Warframe Specters) | Puncture, Viral | Radiation | U36 |
| Sentient | Cold, Radiation | Corrosive | U36 |
| Narmer | Slash, Toxin | Magnetic | U36; Archons in Archon Hunts fixed to Narmer in U36.0.8 |
| The Murmur (includes Rogue Necramechs) | Electricity, Radiation | Viral | U35.5 / U36 |
| Zariman | Void | none | U36 |
| Scaldra | Impact, Corrosive | Gas | U38 |
| Techrot | Gas, Magnetic | Cold | U38 |
| Anarchs | Impact, Electricity | Radiation | U41 |
| Tenno | none (neutral) | none | Damage/Tenno |
| Overguard (layer, any faction) | Void | none | Overguard > Mechanics |
| Object | none; no crits and no status | none | Damage/Object |

Infested Deimos "Vulnerable to Blast and Gas, Resistant to Viral" comes from Damage/Infested_Deimos (U36). Deimos units are also partly **immune** to Viral status (Damage/Viral Damage intro: "with some Deimos units being outright immune"; Deimos Jugulus "Has immunity to Viral status"; all Infested Demolishers are immune to Viral status).

Not covered by the table:
- **Stalker faction** (Stalker, Acolytes): "As of Update 36, Acolytes are neutral to every element" (Acolytes > Tips). Shadow Stalker adapts (section 9).
- **Neutral overrides.** Many enemies have `FactionDamageOverride = ""` in Module:Enemies/data, which the infobox renders as no modifiers. These include all Railjack/space fighters (Exo, Gyre, Kosma, Orm, Taro, Vorac...), drones, Dargyns, Warframe Specters (Excalibur Specter etc.), Infested Hives, Jordas Golem, and Infested Mesa. Some enemies take another faction's modifiers: Corrupted Jackal uses Corpus; Penta Ranger, Quanta Ranger and Rana Del use Grineer; Thrax and Void Angels use Zariman; Death Scuttler and Scarlet Nexifera use The Murmur; Wolf of Saturn Six and Saturn Six Fugitive use Grineer. (Source: `Enemies_data_*.lua` and `Enemies_infobox.lua` line 607, which passes `FactionDamageOverride or Faction` to `DamageTypes.healthMod`.) This conflicts with Damage/Orokin, which says Warframe Specters use the Orokin modifiers; see Open questions.
- Some overrides in the data are internal paths (e.g. Errant Specter `/Lotus/Types/Enemies/Corpus/Gamemodes/PurgatoryWarriorRanged`). These look like data-entry errors; treat them as UNVERIFIED.

### How the modifier combines
- Modifiers from different sources multiply: "Sources of damage type resistance stack multiplicatively" (Damage Type Modifier > Mechanics; U30.5 change to multiplicative "damage remaining").
- External changes can lower a modifier to at most -100% (x0) (Damage Type Modifier > Mechanics).
- Per damage type: `inflicted_type = starting_type x faction_modifier(type)` (Damage/Calculation > Unarmored Enemies, written as `1 + Health-type Modifier`). The faction multiplier applies per damage type, before armor.

Worked example. A Kuva Heavy Gunner (Kuva Grineer) is hit for 100 Heat + 100 Corrosive + 100 Impact. Before armor it takes 100 x 0.5 + 100 x 1.5 + 100 x 1.5 = 350.

---

## 3. Armor

### 3.1 Enemy damage-reduction formula (Update 36 onward)
Source: Armor > Enemy Damage Reduction Formula; Damage/Calculation > Armored Enemies; patch U36 "Altered the formula for Armor Reduction".

```
if NetArmor <= 2700:  DR = 0.9 * sqrt(NetArmor / 2700)
else:                 DR = NetArmor / (NetArmor + 300)        # only under "exceptional conditions"
health damage multiplier = 1 - DR
```

- The two branches meet at 2700: 0.9 x sqrt(1) = 0.9 and 2700/3000 = 0.9.
- The wiki gadget computes the same value as `sqrt(3 * armor) / 100` (`enemyinfoboxslider.js`, `armor_redux`), since 0.9/sqrt(2700) = sqrt(3)/100.
- The Tenno (player) formula is still `Armor/(Armor+300)` and is irrelevant to the calculator.
- **Minimum damage per type:** "When damage is reduced from armor, each damage type has a minimum damage of 1 ... Non-Armor sources of damage reduction do not have such a behavior" (Armor > Enemy DR formula). Example: a Braton (3 damage types) always deals at least 3 to an armored target.

Worked examples (DR → damage multiplier):

| Net armor | DR | multiplier on health damage |
|---|---|---|
| 2700 (cap) | 0.9000 | 0.1000 |
| 2000 | 0.7746 | 0.2254 |
| 500 | 0.3873 | 0.6127 |
| 200 (floor) | 0.2449 | 0.7551 |
| 3000 (exceptional) | 0.9091 | 0.0909 |

### 3.2 Armor cap and floor
Sources: Enemy Level Scaling > Armor; U36 patch notes ("Armor has a maximum cap of 2700 Armor (90% Damage Reduction). Armor has a minimum cap of 200 Armor. Steel Path no longer increases Armor values.").

```
scaled  = floor(BaseArmor * ArmorMultiplier(level))      # floor per enemyinfoboxslider.js
initial = clamp(scaled, 200, 2700)  if BaseArmor > 0 else 0
```
- The 200 floor applies only to the **initial** value. Stripping can take armor below 200 (Enemy Level Scaling > Armor).
- An enemy with base armor 0 stays at 0. The gadget clamps only `if (armor > 0)`.
- The `floor()` comes from the wiki gadget (`Math.floor(base_armor * armor_multi)`). UNVERIFIED that the game floors armor. It matters little, since the DR difference is below 0.02 percentage points.
- Armor strips apply to the **clamped** value. A level-1000 Corrupted Heavy Gunner has raw scaled armor 35,851, clamped to 2700; 80% Corrosive leaves 540, not 7,170 clamped to 2700. This is implied by "clamps an enemy's initial armor" and by the U36 aim of making partial strip useful at high level. It is not stated in those exact words, so it is marked derived.

### 3.3 Armor stripping

Three families exist, and they stack differently. The general formula from the wiki (Damage/Heat Damage > Armor Stripping; Damage/Corrosive Damage; Corrosive Projection > Armor Removal Interactions):

```
remaining_armor = Armor_initial
                * (1 - sum_of_total_based_strips)            # abilities (% of TOTAL) + Corrosive Projection (additive with each other)
                * (1 - HeatStrip(t))                         # Heat status, multiplicative
                * (1 - (0.20 + 0.06 * CorrosiveStacks))      # Corrosive status, multiplicative; 0 stacks -> factor 1
clamped at >= 0
```

Details:

**a) Corrosive Projection aura.** -18% per max-rank aura (-3% per rank, rank 0-5). It is **additive** with other Corrosive Projections (4 x 18% = 72%) and **additive** with ability strips such as Fusion Strike or Terrify. Example from the page: CP + Terrify → 100% - 18% - 60% = 22% remaining. It is **multiplicative** with Heat and Corrosive status (Corrosive Projection > Armor Removal Interactions; numbers since U27.2). Coaction Drift raises aura strength; the Shield page says Shield Disruption "Hard caps at 80% with Coaction Drift". UNVERIFIED whether Corrosive Projection has the same 80% cap.

**b) Warframe ability strips.** Since U32 they are based on the **total** (original) armor: "Casting an ability that strips 50% Armor will reduce their Armor to 50% on the first cast, and 0% on the second cast" (Armor > Patch History U32). Most scale with Ability Strength (e.g. Terrify 60% x Strength, full strip at 168%). Some are permanent (Avalanche, Sonic Boom, Fusion Strike, Reckoning, Pulverize, Tharros Strike); some are temporary (Terrify, Psychic Bolts, Abating Link, Gaze, Seeking Shuriken). Oberon's Smite removes all armor from one target. Mag's Polarize removes a **flat 400 x Strength** armor, after shields. Hildryn's Pillage removes 25% x Strength of shields first, then armor. The full list with percentages is in Armor > Removing Enemy Armor.

**c) Heat status (Ignite).** Strips the **current** armor, multiplicatively. It ramps: every 0.5 s after the proc, armor is multiplied by 0.5^(1/4) ≈ 0.8409. Cumulative strip is 0% / 15.9% / 29.3% / 40.5% / 50.0% at t = 0 / 0.5 / 1.0 / 1.5 / 2.0 s. It does not stack with more Heat procs: "Proccing more Heat status effects within this time frame will not speed up the armor stripping process". When the proc ends, armor returns in steps every 1.5 s over 6 s (50.0 / 40.5 / 29.3 / 15.9 / 0). **Status Duration slows the ramp**: +100% duration means one step per 1 s and full strip after 4 s; -50% means a step every 0.25 s (Damage/Heat Damage > Armor Stripping, edited 2026-11-14).

**d) Corrosive status.** The first stack strips 26%, each further stack 6% more, to 80% at 10 stacks. Formula: `0.20 + 0.06 x stacks`. Each stack lasts 8 s on its own timer, and stack 11 replaces the oldest. Additive within Corrosive, multiplicative with everything else (Damage/Corrosive Damage). Since U36 (when Corrosive became temporary and stack-based).
- **Emerald Archon Shard:** +2 max Corrosive stacks (+3 Tauforged). 14 stacks = 0.20 + 0.84 = 104%, so a full strip; 13 stacks = 98%.
- **Hydroid passive:** enemies Hydroid has damaged take a first Corrosive stack of 50% instead of 26%, so 10 stacks = 50% + 9 x 6% = 104%, a full strip. Corrosive from any source counts (Armor > Hydroid; U34).

**e) Flat strips of base armor.** Shattering Impact (6 base armor per Impact hit) and Amalgam Argonak Metal Auger (6 per single-dagger hit) remove a flat value from **base** armor, which then scales. Sharpened Claws (Kavat) removes 120%. These are the "only non-ability methods to completely remove armor" (Damage/Corrosive Damage). Corrosive Projection is multiplicative with these and "will not cause them to strip armor faster" (Corrosive Projection page).

**Worked examples on capped armor (2700, DR 90%)**, computed with the formulas above:

| Situation | Remaining armor | DR | Damage vs. unstripped |
|---|---|---|---|
| none | 2700 | 0.900 | 1.00x |
| 1 Corrosive stack | 1998 | 0.774 | 2.26x |
| 10 Corrosive stacks | 540 | 0.403 | 5.98x (wiki: "approximately x6") |
| Heat at full ramp | 1350 | 0.636 | 3.64x |
| Heat + 10 Corrosive | 270 | 0.285 | 7.15x |
| Heat + 10 Corrosive + 4x CP | 2700 x 0.5 x 0.2 x 0.28 = 75.6 | 0.151 | 8.49x |
| 1x CP + Terrify at 100% Strength | 2700 x 0.22 = 594 | 0.422 | 5.78x |
| same + 10 Corrosive | 118.8 | 0.189 | 8.11x |

The 1-Corrosive (x2.3) and 10-Corrosive (x6) results match the multipliers quoted on Damage/Corrosive Damage, and 1 CP x1.8 / 4 CP x5.2 match Corrosive Projection > Notes. This cross-checks the sqrt formula.

---

## 4. Shields (enemy)

### 4.1 Faction modifiers and status effects on shields
- Post-U36 the faction modifier applies to shields too (section 2). There is no separate shield damage-type modifier any more. The old Shield/Proto Shield types were removed in U36.
- **Magnetic status**: damage to shields and overguard x `[2 + 0.25 x (stacks - 1)]`, i.e. x2.0 at 1 stack and x4.25 at 10 stacks (max). Each stack lasts 6 s on its own timer. When the shield or overguard breaks, a forced Electricity proc deals 3% of max shield/overguard per Magnetic stack (max 30%) over 6 s. Magnetic also stops shield regeneration (Damage/Magnetic Damage; U36 changes).

### 4.2 Toxin bypass
- Toxin damage and Toxin status go straight to health and skip shields (Shield > Effects; Damage/Calculation > Armored Enemies). Toxin does not bypass overguard.
- **Exceptions**, where Toxin cannot bypass shields: **Treasurer** ("Toxin procs have no effect (dealing 0 damage) while their shields are up"), **Hounds** (Sisters of Parvos; Hound page: "Their shields cannot be bypassed by Toxin damage or procs"). Damage/Calculation names the same two. The full list is UNVERIFIED (see Open questions).
- The wiki also says that shielded enemies' effective HP with Toxin is not simply health + shield, because Toxin damage can drain health while the rest drains shields (Enemy Level Scaling > For Enemies with Health and Shields). The engine must route each damage type to its own layer.

### 4.3 Shield gating on enemies
Source: Shield > Shield Gating (edited 2026-08-23); U27.2 notes ("Foe: Enemies - Corpus in particular - also have received a bit of a Shield Gating").

- When an enemy's shield is fully depleted, a gate lasts **0.1 s**. During it only **5%** of incoming damage reaches health.
- **Weak points (headshots etc.) bypass the gate completely.**
- **Warframe ability damage ignores the gate** (U27.2: "if an instance of Damage from an ability is greater than the Shield value, it will go into health as well").
- Some AoE attacks (e.g. slam attacks) are blocked completely by the gate rather than reduced to 5%.
- Status effects and extra damage instances tied to an attack (e.g. Xata's Whisper) still damage the enemy.
- Enemy Level Scaling > EHP: "In the cases where you are trying to one-shot shielded enemies without Toxin damage, their effective hit-points will actually be higher due to their shield gate mechanic."
- Overguard has **no** gate on enemies (Overguard > Enemy: "Enemies do not have any invulnerability gating when their Overguard is depleted").

Engine note: for sustained DPS the gate affects only the hit that breaks the shield and anything landing in the next 0.1 s. For time-to-kill and one-shot calculations against shielded enemies, damage that overflows the shield from a non-weak-point, non-ability hit should be multiplied by 0.05. UNVERIFIED: whether the overflow of the breaking hit itself is scaled to 5% or discarded. The wiki text "during which only 5% of the damage dealt will damage their health" suggests 5%.

### 4.4 Shield stripping
- **Ability shield strips take a percentage of the current value** and so have diminishing returns ("an ability that removes 50% shields, the first cast will bring shields down from 100% to 50%, and the second cast will bring the 50% down to 25%"). This is the **opposite** of ability armor strips, which use total armor since U32 (Shield > Removing Enemy Shields).
- Shield Disruption aura: -18% per max-rank aura, up to 72% with 4 players, hard cap 80% with Coaction Drift.
- Psychic Bolts 80%, Gaze 50%, Pillage 25% (shield before armor), Rakhali's Cavalry 35% per hit, all x Ability Strength.

### 4.5 Shield recharge (for TTK only)
Since U36: recharge delay depends on accumulated shield damage, with a minimum and a maximum. A break counts as 200% (the maximum delay). Recharge rate scales with enemy level and accelerates while recharging (Enemy Level Scaling, U36 notes). Shield page: "Enemy shields take a maximum of 3 seconds to begin regenerating". Exact rate formulas: UNVERIFIED (not on the wiki). Amalgam shields do not regenerate once fully removed (Amalgam > Notes). Treasurer shields do not recharge except from Shield Ospreys.

---

## 5. Overguard

Sources: Overguard page (edited 2026-10-01); Enemy Level Scaling > Overguard; Eximus.

- **Damage-type modifiers:** neutral, except **Void x1.5** ("a 50% vulnerability from Void damage"). Faction modifiers do not apply to overguard.
- **Not affected by:** armor, pure damage reduction, damage-type resistance effects (Adaptation, Caliban passive), Viral. Toxin does not bypass it.
- **Affected by:** Magnetic (x2.0 → x4.25, same as shields), **Damage Vulnerability** (since U37 several abilities' vulnerability also applies to shields and overguard: Vast Untime, Magnetize, Beguiling Lantern augment, Firing Line, Mass Vitrify, Splinter Storm, Blazing Chakram, Molecular Prime), Assassin Posture and Unleashed (+300% from beasts), Photon Strike (+200%), Secondary Fortifier (up to x8 for secondaries and arch-guns), Smite (removes overguard instantly), Null Audit (-50%).
- **Crowd control:** while overguard is up, the enemy ignores Stagger, Knockdown, Stun, Mind Control, Confusion/Radiation, Slow, Ragdoll, Blind, Lifted. It takes at most **4 Cold stacks**. It still **receives** status effects (DoTs tick); "Prevents status effects and damage spillover to Shields and Health" in Module:DamageTypes/data refers to CC and to damage not passing through. An overguard VIP takes at most **3 Puncture stacks** (U36 fix: "instead of the intended 3"). UNVERIFIED: whether DoT ticks while overguard is up damage the overguard or the health. The wiki implies the overguard, but it is not explicit.
- **Spillover:** the module text says overguard prevents damage spillover to shields and health, while enemies have no overguard gate. UNVERIFIED: whether the overflow of the hit that breaks overguard carries into shield/health (see Open questions).
- **Base overguard:** all Eximus units have base overguard **12** (Enemy Level Scaling > Overguard; gadget `base_overguard = 12`). Overrides exist in data, e.g. one Scaldra unit has `EximusOverguard = 24`. Archwing, Warden and Prosecutor Eximus have no overguard. Other sources: Ancient Protector (9x the protector's health, U38.5: 15 s), Severed Warden, Thrax, Nox, Scaldra Dedicant (helmet or ampule removes it), Kuva Trokarian, Corpus Target enemies, Deep/Temporal Archimedea "Bolstered Belligerents" (overguard = 50% of max health), Void Armageddon Exodamper.
- **Overguard scaling** is in section 7.5.

---

## 6. Health

- Only the post-U36 "Health" class exists. Faction decides the modifiers (section 2).
- Viral status: health damage x `[2 + 0.25 x (stacks - 1)]` (x2.0 → x4.25 at 10 stacks), multiplicative with other bonuses, and works on armored ("yellow") health (Damage/Viral Damage). The hit that applies the first Viral proc does not benefit from it, and multishot pellets of the same shot do not benefit from each other's procs (Damage/Viral Damage).
- The enemy health bar is yellow while armor > 1 (Armor intro: "enemies with more than one point of armor have their health bars displayed yellow").

---

## 7. Level scaling

Source: Enemy Level Scaling (edited 2026-07-29), cross-checked against Module:Enemies/infobox and MediaWiki:Gadget-enemyinfoboxslider.js. Health/shield/armor formulas are from U36 (2024-06-18); the smoothstep was introduced in U27.2.

### 7.1 General form
```
q          = CurrentLevel - BaseLevel            # Overguard: see 7.5 (disputed)
f_i(q)     = 1 + C_i * q^E_i                     # two endpoint curves, i = 1 (low), 2 (high)
t          = clamp((q - L) / (U - L), 0, 1)
s          = t*t * (3 - 2*t)                     # smoothstep
Multiplier = f1 + (f2 - f1) * s
Stat       = BaseStat * Multiplier
```
- Transition bounds: **health, shield, armor L=70, U=80**; overguard L=45, U=50; enemy damage (Corpus/Grineer/Techrot) L=1, U=25.
- The bounds apply to **q (levels above base level)**, not to the absolute level. The gadget implements `trans()` on `curr_lvl - base_lvl`. So a base-level-8 enemy blends between levels 78 and 88.
- **Float32:** the wiki says every intermediate result is a binary32 value, evaluated as `f32(f1 + f32(f32(f2 - f1) * s))`, without rearranging into `f1(1-s) + f2*s` and without substituting f2 when s = 1. At high levels this matters. At q = 992 (Orokin health) f1 = 29,429.32 and f2 = 1,212.5455, and the f32 interpolation gives **1,212.5449**. For overguard at very high levels the error reaches about 1% (section 7.5). Whether the game really works this way is UNVERIFIED by any second source; the wiki states it as the in-game method.
- Level cap 9999, except Void Fissure missions (Enemy Level Scaling intro). Eximus are capped at 9999 too (U25.0.7).
- **Base level** is per enemy (`BaseLevel` in Module:Enemies/data). Examples: Heavy Gunner, Kuva Heavy Gunner and Corrupted Heavy Gunner have 8; Elite Crewman has 1. The inspiration's changelog (guide.txt 2023-06-11) says it assumed 1 for every enemy until then. This is an accuracy trap: the base-level table must be part of the data pipeline.

### 7.2 Health coefficients

| Scaling class | C1 | E1 | C2 | E2 | Applies to (wiki page / module key) |
|---|---|---|---|---|---|
| Grineer, Kuva Grineer, Scaldra | 0.015 | 2.12 | 10.7332 | 0.72 | page "Grineer, Scaldra"; module Grineer, Kuva Grineer, Scaldra |
| Corpus, Corpus Amalgam | 0.015 | 2.12 | 13.4165 | 0.55 | page + module |
| Infested, Infested Deimos | 0.0225 | 2.12 | **16.0998 (page) / 16.100 (module)** | 0.72 | see Open questions |
| Orokin/Corrupted, Anarchs | 0.015 | 2.10 | 10.7332 | 0.685 | page "Anarchs, Corrupted"; module Orokin, Anarchs |
| Techrot | 0.02 | 2.12 | 15.0998 | 0.70 | page + module |
| Default: Murmur, Sentient, Stalker, Narmer(?), Unaffiliated, Zariman(?) | 0.015 | 2.00 | 10.7332 | 0.50 | page "Murmur, Sentient, and Unaffiliated"; module `Default` fallback |

- `FactionScaling` overrides: Jordas Golem uses Default; H-09 Apex (Techrot) and H-09 Efervon Tank (Techrot) use Corpus (Module:Enemies/data).
- The page contradicts itself on Anarchs: it lists them under "Anarchs, Corrupted" and also says "Murmur, Sentient, Anarchs, and Unaffiliated health scales as follows". The module uses the Orokin coefficients for Anarchs.
- Narmer and Zariman are not named on the page; the module falls back to Default. UNVERIFIED.

Reference health multipliers (base level 1, f32, from the formulas):

| Level | Grineer | Corpus | Infested | Orokin | Techrot | Default |
|---|---|---|---|---|---|---|
| 50 | 58.45 | 58.45 | 87.18 | 54.15 | 77.60 | 37.01 |
| 100 | 294.49 | 168.97 | 441.23 | 250.88 | 377.63 | 107.79 |
| 200 | 486.18 | 247.61 | 728.77 | 404.13 | 615.00 | 152.41 |
| 1000 | 1551.30 | 599.96 | 2326.45 | 1218.39 | 1900.62 | 340.24 |

Smoothstep illustration (Grineer health, q = 69..81): q=69 → 119.70 (pure f1); q=70 → 123.38 (s=0); q=72 → 141.67 (s=0.104); q=75 → 191.98 (s=0.5); q=78 → 238.49 (s=0.896); q=80 → 252.74 (s=1, = f2); q=81 → 255.00. f1 and f2 do not intersect; the blend is a steep climb of about 130 multiplier points over 10 levels.

### 7.3 Shield coefficients

| Scaling class | C1 | E1 | C2 | E2 |
|---|---|---|---|---|
| Corpus, Corpus Amalgam | 0.02 | 1.76 | 2.0 | 0.76 |
| Corrupted, Anarchs | 0.02 | 1.75 | 2.0 | 0.75 |
| Grineer, Sentient (and Default) | 0.02 | 1.75 | 1.6 | 0.75 |
| Techrot | 0.02 | **1.76 (page) / 1.75 (module)** | 3.5 | 0.76 |

- Module bug: the shield table's key is `"Corrupted"` while Corrupted enemies have `Faction = "Orokin"`, so the wiki slider uses Default shields (C2 = 1.6) for Corrupted units. The page says 2.0. Trust the page, but it is UNVERIFIED in game.
- U36: "Shields now scale more quickly starting at level 80." At level 2025 and above, shields grow faster than health (Enemy Level Scaling > Shielding Ratio).

### 7.4 Armor coefficients (all factions)
`C1 = 0.005, E1 = 1.75, C2 = 0.4, E2 = 0.75`, then clamp to [200, 2700] (section 3.2). Multipliers at base level 1: lvl 50 → 5.54, lvl 100 → 13.55, lvl 200 → 22.19, lvl 1000 → 72.08. An enemy with base armor 200 reaches the 2700 cap at multiplier 13.5, about level 100. Base 500 reaches it at multiplier 5.4, about q = 49. Every armored non-boss enemy is at 2700 from roughly level 60-130 upward.

### 7.5 Overguard scaling
`C1 = 0.0015, E1 = 4, C2 = 260, E2 = 0.9`, smoothstep L=45, U=50.

**Conflict on q:** the Enemy Level Scaling text says overguard uses `q = CurrentLevel - 1` and ignores base level. The wiki's own gadget uses `curr_lvl - base_lvl`. For an Eximus Heavy Gunner (base level 8) at level 58 this means **118,710** (q=57) versus **105,506** (q=50), a 12.5% difference. The page text was edited more recently (2026-07-29 edit, f32 rewrite) than the gadget logic. Prefer `level - 1`, but it stays an Open question.

Float32 sensitivity: at q = 999, f32 gives a multiplier of 130,176 against 130,192.4 in f64. At q = 9998 it gives 1,048,576 against 1,034,893 (+1.3%), because f1 ≈ 1.5e13 swamps float32 precision.

### 7.6 Enemy damage scaling (for completeness, not used for player DPS)
Default `1 + 0.015 q^1.55`. Corpus, Grineer and Techrot blend `f1 = 1 + 0.015 q^1.75` and `f2 = 1 + 0.0075 q^1.55` over L=1, U=25, then x2 (U36.1). Infested x3.

---

## 8. Mission and unit modifiers

### 8.1 Steel Path
Sources: The Steel Path (rules section; U36 notes); Enemy Level Scaling U36; gadget.
- **Level +100** (+50 in Archwing/Empyrean/Railjack; Sealab underwater Archwing stays +100; Duviri landscape +20 with **no** health or shield change; the Undercroft, Circuit and Isleweaver use standard SP). Steel Path bounties start at level 100. Eidolons go to level 110 / 120 / 130.
- **Health x2.5, Shield x2.5.** U36 fixed shields, which were applied twice (x6.25).
- **Armor: no Steel Path multiplier since U36.** The Steel Path page still says the Undercroft (U33) and Simulacrum (U35.5) apply "+250% Health, Armor, and Shields". Both lines predate U36 and are outdated.
- **Order** (gadget): `base_health = (BaseHealth + SteelPathHealthBonus)`, then the Eximus base increase (8.2), then **x2.5**, then the level multiplier. The 2.5 is a multiplier on the base and so commutes with the level multiplier, apart from f32 rounding.
- **Flat Steel Path base-health bonuses** (U40 patch notes / Module:Enemies/data `SteelPathHealthBonus`): Juggernaut +1,800; Juggernaut Behemoth +3,000; Necramech/Juggernaut Demolisher +2,000; Legacyte +7,500 (data comment: "Doesn't seem to have this in practice"); The Fragmented +150,000; Anarch Commandeered Primes +10,000; two Sentient bosses +440,000 / +170,000. UNVERIFIED: whether the bonus is added before the x2.5 (gadget) or after.
- **Overguard:** neither the wiki page nor the gadget applies x2.5 to overguard. UNVERIFIED.
- The Steel Path level bonus is **included** in the displayed enemy level. A "level 1000 Steel Path" enemy uses level 1000 in the formulas; do not add another 100.

### 8.2 Eximus
Source: Enemy Level Scaling > Health/Shields tabs "Eximus"; gadget implementation.

Eximus units use their faction's curves but get a level-dependent increase to **base** health and shield before scaling. `d = level - BaseLevel` (the gadget uses `lvl_diff`; the page writes `x` without defining it, see Open questions). With `k(d)`:

| d | k(d) |
|---|---|
| d ≤ 15 | 1 |
| 15 < d ≤ 25 | 1 + 0.025 (d - 15) |
| 25 < d ≤ 35 | 1.25 + 0.125 (d - 25) |
| 35 < d ≤ 50 | 2.5 + (2/15)(d - 35) |
| 50 < d ≤ 100 | 4.5 + 0.03 (d - 50) |
| d > 100 | 6 |

```
c = 0.25  if enemy has shields or armor, else 0.375
EximusBaseHealth = max(BaseHealth * 1.1, c * (BaseHealth + 900) * k(d))
EximusBaseShield = max(BaseShield * 1.1, BaseShield * k(d))          # d<=15: BaseShield * 1.1
```
- Codex health values are **not** the Eximus base (Enemy Level Scaling). The data has `EximusHealth`/`EximusShield`/`EximusArmor` overrides that replace the base first (gadget).
- Armor: no Eximus multiplier (only the `EximusArmor` override).
- Overguard: base 12 (section 5), scaled as in 7.5.
- U32 raised Eximus base health by about 30% and the mercy threshold to 80%.

### 8.3 Other multipliers (from the gadget, for completeness)
- Archimedea "Empowered": health and shield x(2.0 + 0.5 x players), plus `ArchimedeaHealthBonus` added to base.
- "Icebind": health, shield **and armor** x(3 + players). A multiplied armor still clamps to 2700 in the gadget.
- Bolstered Belligerents: overguard = 50% of max health (Overguard page). U38.5: Techrot Babau gets 75% less.
- **Squad-size scaling:** normal enemies do not scale with squad size. Exceptions are Demolishers (+50% / +100% / +200% health with 2/3/4 players; Demolisher page), Acolytes and Archons (Enemy Level Scaling intro; amounts UNVERIFIED).

---

## 9. Special enemies: damage reduction and attenuation

### 9.1 Damage attenuation (general, U40 2025 rework)
Source: Damage Reduction page, U40 / U40.0.2 / U40.0.3 patch notes (DE's own text).

- **MDPI** (max damage per instance) and **MDPS** (max damage per second) caps. Since U40 both are **proportional to the enemy's max health**, using the health value only (not shield, armor or overguard). MDPS applies **per player** since U40; MDPI always did.
- Damage modifiers apply **before** attenuation (U34 Archon notes: "Modifiers now apply before Damage Attenuation").
- **Per-hit attenuation formula** (Hemocyte > Damage Reduction; the same structure is shown for Lephantis with `MaxPeakDamageHealthPercent`):
```
IncomingDamage  = Damage * BodyPartMultipliers * CritMultiplier(if crit) * other multipliers
MaxDamagePerHit = p * MaxHealth
Attenuated      = IncomingDamage * MaxDamagePerHit / (IncomingDamage * Multishot + MaxDamagePerHit)
```
  Worked example (wiki): 1,000 damage, 3x crit, 2x weak point, multishot 2, against a level-117 Hemocyte (max health 2,472,207, p = 5%): Incoming = 12,000; MDPH = 123,610; Attenuated = 12,000 x 123,610 / (24,000 + 123,610) = **10,049**.
- The DPS-cap (MDPS) model is **not documented on the wiki** ("MaxDamagePerSecondHealthPercent not formulated yet"). The Archon page gives pre-U40 parameters: MDPT = 460,000, MDPS = 935,000, exponential decay λ = 2/3, ShotgunCompensation = 0.7 if base multishot > 0. The formula they plug into is not shown, and U40 replaced the fixed values with health-proportional ones. UNVERIFIED.
- **Enemies with standardized attenuation after U40.0.2** ("only true Bosses / HUD health bar enemies"): Acolytes (base health 5,500, base shields 2,500; can be one-shot), Adversaries (Kuva Lich, Sisters of Parvos, Technocyte Coda; U40.0.3 raised their MDPI), Archons, Eidolons, Infested Oni (one-shottable), Lephantis and Hemocyte, Legacyte, Mocking Whispers, Oraxia (Isleweaver), Protea (Deadlock Protocol), Technocyte Coda Duets, The Fragmented, Void Angel, Zealoid Prelate, Kuva Lich Thralls, Juggernaut variants.
- **Attenuation removed in U40.0.2:** Scaldra Dedicants, Demolishers (including Demolisher Necramechs), Deimos Jugulus, Deimos Saxum/Rex, Rogue Necramechs (Voidrig, Bonewidow), Amalgams, Empyrean units (Aurax, Numon, Vambac, Zerca), Errant Specters, Gruzzling, Necramite, Sister of Parvos Hounds, Techrot Babau, Treasurer, Tusk Thumpers (all). Skittergirl: attenuation removed in U40.0.3 and replaced by 90% DR, base health 600.
- Base-health changes in U40 that the data must reflect: Treasurer 100 → 2,500; Tusk Thumper 3,000 → 9,000 (Bull 12,000, Doma 15,000; plates 500); Rogue Necramech 3,800 → 7,500 (limbs 600); Necramech Demolisher 3,000 → 4,500; Juggernaut 2,700 → 900 (Behemoth 4,500 → 1,500); Lephantis head 900, body 3,000; Hemocyte 5,000; Scaldra Dedicant 2,000; Deimos Saxum 750; Techrot Babau 1,500 → 10,000 (U40.0.3); Eidolons, for example Teralyst body 50,000, radiator 25,000, shields 50,000; Stalker 750, Shadow Stalker 950, Protector Stalker 1,100 health + 500 shields + 2,000 overshields.

### 9.2 Per-enemy notes

| Enemy | Mechanic | Numbers | Source / freshness |
|---|---|---|---|
| **Archons** (Amar, Boreal, Nira) in Archon Hunts | attenuation + flat DR | 20% flat DR to all damage, 50% DR to status damage; at most 4 stacks of any status; armor cannot be removed (Boreal page: "has armor which cannot be removed"); Narmer modifiers (U36.0.8); level 150 | Archon (2026-07-20), but its MDPT/MDPS values are pre-U40 |
| **Eidolons** (Teralyst, Gantulyst, Hydrolyst) | attenuation on shield and health | MDPH = 200% of max health, MDPS = 100% of max health; body part multiplier 2x; crit damage bonus 2x; head 3x (AoE gets neither the head bonus nor the crit bonus on the head) | Eidolon Teralyst (2026-10-04); U40.0.3 "Greatly increased the MDPI" |
| **Lephantis** | per-hit attenuation | p = 5% of max health; crit damage x2; Grineer head chest weak point x2; level 30 (SP 125) | Lephantis |
| **Hemocyte** | per-hit + DPS attenuation | MDPH 5%, MDPS 20% of max health; crit x2; chest weak point x2 | Hemocyte |
| **Juggernaut** | flat DR | 90% DR, +50% DR against AoE and melee; stomach and back maw weak points x10 damage and x4 crit damage (status only through weak points); other parts 2x crit. Demolisher Juggernaut: 20% DR instead of 90%; boils x0.5 with no 2x crit | Juggernaut (2026-10-06) |
| **Demolishers** | squad health | +50/100/200% health for 2/3/4 players; immune to most abilities (nullifier pulse every 5 s); do not freeze at 10 Cold stacks (slow 90%); all Infested Demolishers immune to Viral status, Demolisher Juggernaut and Charger immune to Viral and Toxin **damage** | Demolisher |
| **Demolisher Bursa** | flat DR | 20% | Bursa |
| **Necramechs (Rogue/Demolisher)** | status/CC | immune to Knockdown, Stagger and status effects; Murmur modifiers for Rogue Necramechs | Necramech, Damage/The Murmur |
| **Orphix** | DPS-based DR, no crits in the DPS term | `DPS = damage x fire rate x multishot x body part` (quantized, with health modifiers); multiplier: 1 (DPS ≤ 1000); 0.7 + 300/DPS (≤ 2500); 0.5 + 800/DPS (≤ 5000); 0.2 + 2300/DPS (≤ 10000); 0.02 + 4100/DPS (above); crits applied after | Orphix (2026-08-13; not in the U40 list, so presumably still current) |
| **Condrix** | DPS-based DR like Orphix | formula not documented | Condrix |
| **Treasurer** | flat DR + shield | shield 99% DR, not bypassed by Toxin, does not recharge (except from Shield Osprey); health 98% DR with no armor. U40 base health 2,500 and U40.0.2 attenuation removed, but the page was last edited 2025-11-01, so the 99%/98% values may be outdated | Treasurer |
| **Sentients** (most) | adaptation | gates at 25/45/65/80% health lost; at each gate they adapt to the type they took most of. Resistances 90%, 80%, 75%, 70% for the 1st-4th adapted type, **plus** a separate 40% resistance per adaptation that stacks multiplicatively and cannot be removed. Per-hit cap: 25% of max health at full health; 20% in 75-55%; 20% in 55-35%; 15% in 35-20%; no cap below 20%. Cannot adapt to True, Finisher, Cinematic (Slash bleed). Removable by Void Beam, Radial Howl, Paracesis rank 40, Shedu pulse, Xata's Whisper (the 40% part stays) | Sentient (2026-11-17) |
| **Shadow Stalker** | adaptation | same gates, **95%** resistance per adapted type; also removed by The Vast Untime | Shadow Stalker |
| **Acolytes** | attenuation, status cap | at most 4 stacks of any status, Impact at most 3; Radiation only amplifies ally damage; neutral faction modifiers; beam weapons weakest, AoE strongest (an attenuation effect) | Acolytes |
| **Kuva Lich Thralls** | flat DR | 25% DR on top of armor | Damage Reduction |
| **Prosecutors** (Guardsman Eximus) | aura modifiers | allies get a -100% damage-type modifier against physical and primary elements and -85% against combined elements, but +100% against the Prosecutor's own element; stacks multiplicatively with other modifiers and between Prosecutors; Void damages any Prosecutor; 10x base health instead of overguard | Guardsman, Eximus |
| **Guardian Eximus** | DR | **conflict:** Eximus page: three shields give 90% DR **to AoE attacks** for itself (and AoE DR applies to its own overguard, not to allies' overguard). Damage Reduction page: 90% damage-type resistance to all types for itself and allies. Only Corpus, Murmur (Necramech) and Techrot grounded units | Eximus (2026-11-16, newer) vs Damage Reduction |
| **Hyekka / Hyekka Master** | type resist | 80% Heat resistance | Damage Reduction |
| **Raknoids** | flat DR | Aurax Atloc 80%; Kyta 25% on shields only; Scyto 50% | Damage Reduction |
| **Deimos Jugulus** | regen, immunities | regenerates 2% of max health per second after 10 s without damage; immune to Viral status; attenuation removed U40.0.2 | Deimos Jugulus |
| **Amalgams** | shields, sub-faction | Corpus Amalgam modifiers; shields never regenerate once fully removed; Alkonost immune to status; attenuation removed U40.0.2 | Amalgam |
| **Ancient Disruptor** aura | ability DR | 90% DR against ability damage | Damage Reduction |
| **Grustrag Bolt** | DR | 50% against Grineer | Damage Reduction |

---

## 10. Worked example: Corrupted Heavy Gunner, level 1000, Steel Path

Base stats (Module:Enemies/data/orokin): Faction Orokin; Health 700; EximusHealth 700; Armor 500; no shields; BaseLevel 8; head 3.0x.
Level 1000 already includes the Steel Path +100, so q = 1000 - 8 = 992, which is past the transition (s = 1).

**Health** (Orokin curve: C1 0.015, E1 2.10, C2 10.7332, E2 0.685):
- f1 = 1 + 0.015 x 992^2.1 = 29,429.32; f2 = 1 + 10.7332 x 992^0.685 = 1,212.5455.
- Multiplier (f32, order kept) = **1,212.5449**.
- Steel Path base = 700 x 2.5 = 1,750, so **health = 1,750 x 1,212.5449 = 2,121,953.5**.
- (Normal path at level 1000 would be 848,781.4.)

**Armor:** multiplier = 1 + 0.4 x 992^0.75 = 71.704; raw = floor(500 x 71.704) = 35,851, **clamped to 2,700**. Steel Path does not change it. DR = 0.9 x sqrt(2700/2700) = **90%**.

**Effective health against a neutral damage type, no strip:** 2,121,953.5 / 0.1 = **21,219,535**.
- Puncture or Viral (Orokin vulnerable x1.5): EHP for that type = 14,146,357.
- Radiation (x0.5): 42,439,070.
- With 10 Corrosive stacks (armor 540, DR 0.4025): 2,121,953.5 / 0.5975 = 3,551,282 for neutral damage.

**If it spawns as an Eximus:** the unit has armor, so c = 0.25; d = 992 > 100, so k = 6.
- Eximus base health = max(700 x 1.1, 0.25 x (700 + 900) x 6) = 2,400.
- Steel Path: 6,000. Health = 6,000 x 1,212.5449 = **7,275,270**.
- Overguard = 12 x M_OG: with q = level - 1 = 999, M = 130,176 (f32) → **1,562,112**. With q = level - base = 992 → 1,552,896. Steel Path factor on overguard: none according to the wiki (UNVERIFIED).
- Armor 2,700 (no Eximus change).

Other worked examples:
- **Elite Crewman** (Corpus, H 110, S 150, BL 1), level 200 Steel Path (q = 199): health multiplier 247.6087, shield multiplier 112.7270 → health 275 x 247.6087 = **68,092**, shield 375 x 112.7270 = **42,273**. As an Eximus: base health max(121, 0.25 x 1010 x 6) = 1,515, health = 1,515 x 2.5 x 247.6087 = 937,818; base shield 900, shield 253,636.
- **Eximus Heavy Gunner** (Grineer, H 300, A 500, BL 8), level 58, normal path (d = 50, k = 4.5): base health max(330, 0.25 x 1200 x 4.5) = 1,350; Grineer multiplier at q = 50 is 60.9663, so health = **82,305** (non-Eximus: 18,290). Armor floor(500 x 5.70) = 2,850, clamped to 2,700. Overguard 118,710 (q = 57) or 105,506 (q = 50).
- Formula cross-check: the wiki's Hemocyte example gives a level-117 max health of 2,472,207 = 5,000 x (1 + **16.1** x 116^0.72). The page's own calculator, using 16.0998, gives 2,472,177. The coefficient conflict therefore shows up inside the wiki itself.

---

## 11. Edge cases

1. **Base level, not level, drives scaling.** q = level - BaseLevel, and the 70/80 smoothstep window is in q. Enemies with base level 8 or higher blend later than level 70-80.
2. **Overguard q is disputed** (level - 1 on the page, level - base level in the wiki gadget). The difference is up to about 12% at mid levels.
3. **Float32 evaluation order** changes results at high levels. Overguard at level 9999 differs by about 1.3% between f32 and f64. Health interpolation must not be simplified to f2 when s = 1.
4. **Armor floor of 200** applies to initial armor only. Strips can go below 200. Enemies with base armor 0 stay at 0.
5. **Armor above 2700** switches to the `A/(A+300)` formula. This happens only in exceptional cases (Icebind-style base-armor multipliers before the clamp, possibly special bosses). The standard engine clamp makes it unreachable from level scaling.
6. **Minimum 1 damage per damage type** applies after armor only, not after other damage reduction. Weapons with many damage types have a larger floor against capped armor.
7. **Ability armor strips use total armor and are additive with Corrosive Projection. Heat and Corrosive are multiplicative with everything.** Shield strips by abilities use **current** shields (diminishing returns), the opposite of armor.
8. **Heat strip ramps** over 2 s in four steps, and Status Duration slows it. A DPS model that assumes an instant 50% overstates early damage.
9. **Corrosive can reach a full strip** only with the Emerald Archon Shard (14 stacks) or the Hydroid passive (first stack 50%).
10. **Flat base-armor strips** (Shattering Impact, Metal Auger) act on base armor before level scaling, so 6 points of base armor are worth far more than 6 points at high level.
11. **Steel Path no longer multiplies armor** (U36). Older pages and old tools apply x2.5 armor; the Undercroft and Simulacrum notes on the Steel Path page are outdated.
12. **Steel Path flat base-health bonuses** exist for some bosses (Juggernaut, Demolishers, Fragmented, Anarch Commandeered Primes, Sentient bosses). The order relative to the x2.5 is unverified.
13. **Eximus health depends on whether the unit has shields or armor** (0.25 vs 0.375 coefficient) and uses the level difference. The Codex health is not the Eximus base.
14. **Toxin does not bypass Treasurer or Hound shields**, and does not bypass Overguard. Toxin procs on Treasurer shields deal 0.
15. **Shield gate:** only 5% leaks for 0.1 s after a break. Weak-point hits and ability damage bypass it; some AoE (slams) is fully blocked. Overguard has no gate on enemies.
16. **Magnetic and Viral** use `2 + 0.25(n-1)`. Viral never affects shields or overguard; Magnetic affects shields and overguard. A hit does not benefit from the Viral or Magnetic proc it applies itself.
17. **Overguard ignores faction modifiers** (except Void x1.5), armor and DR, but not Damage Vulnerability (U37 extended several vulnerabilities to shields and overguard).
18. **Sub-factions differ from the parent:** Kuva Grineer resist Heat, Corpus Amalgam resist Blast, Deimos Infested resist Viral and some are status-immune.
19. **Faction-modifier overrides:** Corrupted Jackal takes Corpus modifiers despite being Corrupted; many Specters, drones and space units are neutral; Thrax and Void Angels use Zariman. Bane-mod faction and modifier faction can differ (UNVERIFIED which one Bane mods follow).
20. **Shield scaling key bug in the wiki module** (Corrupted units get Grineer shield coefficients in the wiki slider). Do not copy the module blindly.
21. **Sentient adaptation** is per damage type and per gate, with a non-removable 40% layer on top of the 90/80/75/70%, plus per-hit caps that prevent one-shots. Sentients cannot adapt to True, Finisher or Cinematic (bleed).
22. **Damage attenuation** applies after all modifiers, uses max health (not overguard, shield or armor), and the per-hit formula divides by `Incoming x Multishot + Cap`, which penalizes multishot. Enemy lists changed in U40 / U40.0.2.
23. **Status caps on special enemies:** Archons and Acolytes take at most 4 stacks (Acolytes at most 3 Impact); overguarded enemies take at most 4 Cold stacks and VIPs with overguard at most 3 Puncture stacks.
24. **Demolisher health scales with squad size;** regular enemies do not.

---

## 12. Open questions

1. Overguard level input: `level - 1` (Enemy Level Scaling text) or `level - BaseLevel` (wiki gadget)? UNVERIFIED.
2. Infested health C2: 16.0998 (page) or 16.1 (Module:Enemies/infobox; also the Hemocyte example value)? UNVERIFIED.
3. Techrot shield E1: 1.76 (page) or 1.75 (module)? UNVERIFIED.
4. Health curve for Narmer and Zariman (and Stalker): the module falls back to Default (0.015, 2, 10.7332, 0.5); the page does not name them. UNVERIFIED.
5. Anarchs health curve: the page lists them both with Corrupted and with Murmur/Sentient/Unaffiliated; the module uses the Orokin coefficients. UNVERIFIED.
6. Does the game really evaluate scaling in float32 in the documented order? Only the wiki says so.
7. Is the Steel Path x2.5 applied to overguard? Neither the wiki text nor the gadget applies it. UNVERIFIED.
8. Is the Steel Path flat health bonus added before the x2.5 (gadget) or after?
9. Eximus formula variable `x`: level difference (gadget) or absolute level? The page does not define it.
10. Does damage from the hit that breaks an enemy shield gate overflow at 5% or get discarded? Does overflow past overguard reach shield or health ("prevents damage spillover" in Module:DamageTypes/data vs no gate on enemies)?
11. Do DoT ticks (Slash, Heat, Toxin, Electricity, Gas) on an overguarded enemy damage the overguard? Bleed is Cinematic; does it bypass overguard like it bypasses armor?
12. Does True damage bypass overguard?
13. Complete list of enemies whose shields Toxin cannot bypass (beyond Treasurer and Hounds). The Enemy template has a `protoshield` parameter that likely marks them, but no list was extracted.
14. Warframe Specters: the Damage/Orokin page says Orokin modifiers; the data has `FactionDamageOverride = ""` (neutral). Which is right?
15. Guardian Eximus: 90% DR to AoE only (Eximus page) or 90% to all damage types for itself and allies (Damage Reduction page)?
16. U40 attenuation: exact MDPI and MDPS percentages per enemy (only Eidolon 200%/100%, Lephantis 5%, Hemocyte 5%/20% known) and the MDPS model itself (decay constant, shotgun compensation). The Archon page values (460k/935k) predate U40.
17. Treasurer DR values (99% shield, 98% health): still valid after the U40 health rework? The page was last edited 2025-11-01.
18. Acolyte and Archon squad-size scaling factors.
19. Does Corrosive Projection have a strip cap with Coaction Drift, like Shield Disruption's 80%?
20. Does the game floor scaled armor to an integer (gadget does)? Are health and shield rounded? The gadget rounds the display to 0.01.
21. Enemy shield recharge delay and rate formulas (U36), needed only for TTK simulation.
22. Kuva Lich, Sister and Technocyte Coda health per rank and level (U40 changed the multipliers); not researched here.
