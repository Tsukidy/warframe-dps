Status: draft

# Weapon damage changed from outside the weapon: Warframe abilities, Helminth, auras, Archon shards, Focus, Warframe arcanes, debuffs and squad buffs (game-truth side)

Research date 2026-11-23 (game state late 2026). Sources are wiki.warframe.com pages read as raw wikitext and cached in `research/external/cache/wiki/` (file name = page name, `/` → `__`). Pages fetched for this file and added to that cache: `Violet_Archon_Shard`, `Topaz_Archon_Shard`, `Crimson_Archon_Shard`, `Focus__Unairu`, `Focus__Zenurik`, `Focus__Vazarin`, `Electric_Shield`, `Parasitic_Link`, `Sonar`, `Metamorphosis`, `Redline`, `Kinetic_Plating`, `Wrathful_Advance`, `Breach_Surge`, `Reservoirs`, `Warcry`, `Grave_Spirit`, `Prismatic_Gem`, `Valence_Formation`, `Plunder`, `Absorb`, `Navigator`, `Helminth`, `Terrify`, `Pillage`, `Mach_Rush`, `Garuda__Abilities__Passive`, `Vauban__Abilities__Passive`, `Kullervo__Abilities__Passive`, `Aura`, `Steel_Charge`, `Rifle_Amp`, `Pistol_Amp`, `Shotgun_Amp`, `Dead_Eye`, `Holster_Amp`, `Worthy_Comradery`, `Mecha_Empowered`, `Empowered_Blades`, `Growing_Power`, `Power_Donation`, `Shepherd`, `Infested_Impedance`, `Shield_Disruption`, `Coaction_Drift`, `Tenacious_Bond`, `Charm`, `Blessing`, `Molecular_Prime`, `Blazing_Chakram`, `Petrify`, `Strangledome`, `Magnetize`, `Rest_%26_Rage`, `Razorwing`, `Paralysis`, `Mass_Vitrify`, `Containment_Wall`, `Reap`, `Theorem_Demulcent` (empty), `Arcane_Rise` (empty), `Arcane_Primary_Charger`, `Arcane_Awakening`, `Arcane_Tempo`, `Magus_Accelerant`, `Arcane_Bellicose`, `Maximization`. Pages already cached by other agents were reused (Roar, Eclipse, Xata's Whisper, Toxic Lash, Nourish, Vex Armor, Elemental Ward, Covenant, Corrosive Projection, Archon Shard, Emerald Archon Shard, Focus/Madurai, Focus/Naramon, arcane pages, Universal Weapon Bonuses, Buffing Fields, Extra Hit, Damage Vulnerability, Armor, Faction Damage Bonus, Amp (Ability), Metronome, Shooting Gallery, Symphony of Mercy, Cat's Eye, Vigorous Swap, Excalibur and Valkyr passives). The machine-readable wiki modules `Module:Ability/data` and `Module:Arcane/data` (cached as `research/external/cache/poc/out/*.json`) were also read; see §13 for why the ability **stats** module must not be trusted.

"U" = game Update. "Str" = the caster's Ability Strength as a multiplier (100 % = 1.0). Numbers are max rank unless stated.

This file is the game-truth counterpart of the inspiration's "external buffs" panel. Overlap with `research/findings/wiki-additive-vs-multiplicative-edge-cases.md` §4-6 is intentional; the values here were re-read from the source pages and agree with that file except where §11 says otherwise.

---

## 0. Bucket model used in this file

Per-hit damage (details in `wiki-core-damage-formula.md`); every external buff below is assigned to exactly one of these slots:

```
hit = Base
    × (1 + ΣBASE)                                  # BASE: Serration-type, additive
    × (1 + ΣPHYS_i or ΣELEM_j per type)            # PHYS / ELEM: per-type, additive inside each type
    × Π(1 + MULT_k)                                # MULT: each independent separate multiplier
    × (1 + ΣFACTION)                               # FACTION: Bane mods + Roar + Damage Blessing, applied again to DoT
    × CritTierMult(CC_rel, CC_abs, CD_rel, CD_abs, CD_mult)
    × WeakPoint/BodyPart × SONAR
    × Π(target VULN) × armor/resistances
+ ExtraHits(XH)                                    # separate damage instances computed from the finished hit
```

- `CC_rel` = critical chance bonus that multiplies base CC (Point Strike, Crimson shard, Hot Shot, Topaz shard). `CC_abs` = percentage points added after (Arcane Avenger, Covenant, Wrathful Advance, Cat's Eye). Smeeta Charm is neither: it is a floor (§8).
- `CD_rel` = adds to the crit-damage mod sum (Vital Sense, Crimson/Violet shard, Grave Spirit). `CD_abs` = "Final Critical Multiplier" added to (CDM − 1) per tier: `1 + tier × (CDM − 1 + CD_abs)` (Arcane Crepuscular, Tenacious Bond). `CD_mult` = buffing fields that multiply crit damage (Electric Shield, Energy Shell, Assimilate, Mandonel).
- Ability Strength: unless stated, a strength-scaled buff value is `listed × Str`, `Str = 1 + Σ(additive strength bonuses)` (Intensify, Molt Augmented, Arcane Crepuscular, Crimson shard, Growing Power, Power Donation, Archon/Umbral Intensify, Focus Sling Strength... are all additive with each other: `Molt Augmented`, `Arcane Crepuscular`, `Archon Shard` crimson row, `Growing Power` § Notes). Parasitic Link is the one **multiplicative** strength source found (§1.15). Power Donation note "Can be used with Overextended to achieve 10 %" implies a floor of 10 % Strength (UNVERIFIED).
- Strength is applied at the **caster**, so for squad buffs the value depends on the caster's strength, not the shooter's.

---

## 1. Warframe ability buffs

### 1.1 Rhino – Roar (inspiration: yes, "Roar")

- Value: `10 / 15 / 25 / 50 %` by ability rank, `× Str`. Subsumed (Helminth): `2 / 9 / 15 / 30 %` `× Str`. Source: `Roar` § info and § helminth; `Universal Weapon Bonuses` § Faction Damage Bonus.
- Bucket: **FACTION** ("considered Faction Damage Bonus, additive with other sources of Faction Damage, and multiplicative with other types of bonus"). Applies to all weapons and to all ability damage. Applies to the caster and to allies in 25 m **at cast time** (they keep it if they leave). 30 s, not recastable while active (except with Piercing Roar). Multiple Roars do not stack; the stronger overwrites; a weaker one cast later can outlast it.
- DoT: like any faction bonus it is applied a second time to DoT ticks (double dip); with Extra Hits it is applied twice on the extra hit and three times on its DoT (`Toxic Lash`, `Extra Hit`).
- Worked example (`Roar` § info): 100 base, Serration (+165 %), Bane of Grineer (+30 %), Intensify (Str 1.3), Grineer target:
  `100 × 2.65 × (1 + 0.5 × 1.3 + 0.3) = 100 × 2.65 × 1.95 = 516.75`.
  Slash tick from that hit: `0.35 × 100 × 2.65 × 1.95 × 1.95 = 352.68` per tick.
- Last changed: subsumed value is "Diminished Damage Increase" (Helminth page row for Rhino); date of the 30 % figure UNVERIFIED (Helminth introduced in U29).

### 1.2 Mirage – Eclipse (Solar) (inspiration: yes, "Eclipse")

- Value: `50 / 100 / 150 / 200 %` `× Str`. Subsumed: `2 / 9 / 15 / 30 %` `× Str` since **U35.5** (patch note on `Helminth`: "Solar Eclipse has a base 30 % increased Weapon Damage. This was previously 150 %"). U35.5 also made both Eclipse modes guaranteed (tap = Lunar DR, hold = Solar damage), removing the light-level dependence.
- Bucket: **MULT** (separate multiplier, "multiplicative with almost all other flat damage buffs"). Exception: weapons whose Condition Overload-type bonus is **Adding** (additive CO): the CO part is computed separately and added, so Eclipse does not multiply it (`Eclipse` § info). Applies **twice** on Exodia Contagion and on glaive explosions. Self only; companions and Hall of Mirrors clones only with Total Eclipse.
- DoT: as a weapon-damage multiplier it is part of the hit's modded damage and therefore scales DoT seeds once (not double-dipped like faction). Whether it is also omitted from the DoT seed for Adding-CO weapons: UNVERIFIED.
- Worked example (`Eclipse`): 250 base, Serration, Intensify: `250 × 2.65 × (1 + 2.0 × 1.3) = 250 × 2.65 × 3.6 = 2385`.
- With Adding CO (from `wiki-additive-vs-multiplicative-edge-cases.md` §2): 100 base, Serration, CO +240 %, Eclipse ×3: `100 × (2.65 × 3 + 2.4) = 1035`, not `100 × 5.05 × 3 = 1515`.

### 1.3 Xaku – Xata's Whisper (inspiration: yes)

- Value: `17 / 23 / 23 / 26 %` `× Str` of the hit's damage, dealt as an **Extra Hit** of Void damage. Duration 35 s. Helminth version: same numbers (no "Diminished" row; only the damage-buff slot restriction). Source: `Xata's Whisper`; `Extra Hit`.
- Formula (`Extra Hit` § Damage Calculation):
  `XH = WeaponHit × p × (1 + ΣFACTION)` where `WeaponHit` already contains base, elemental, crit, faction, body part and external multipliers. Xata is "the only extra hit that does **not** have a forced 1× weak-point multiplier", i.e. it double-dips body part / headshot as well as faction (`Extra Hit` § Notes; `Xata's Whisper` § info).
- The Void hit is then reduced by the target's armor and Void modifiers (Void is neutral since U29.1; Eidolons, Amalgams and some bosses still resist). It is a separate damage instance: it does not dilute the weapon's elements or status weights. It rolls Void status with the weapon's modded status chance. It is reduced by Ancient Disruptor auras / ability-damage reduction (unlike other weapon buffs).
- It cannot crit by itself (listed as a bug on `Extra Hit`; the crit is already inside `WeaponHit`). Covenant headshots and Concentrated Arrow can make it crit (bug list on `Xata's Whisper`).
- Triggers: every multishot pellet; Acid Shells, Concealed Explosives; **Blast detonations** (bug: only Xata does this; then faction applies a third time and elemental bonuses are applied although Blast detonations normally ignore them, `Xata's Whisper` § tips). Not triggered by Bursting Mass or Pathocyst maggots. Does not give melee/sniper combo.
- DoT: Void status is not a DoT, so none.
- Worked example: 100 base, Serration, yellow crit at 2.0×, Bane +30 % on Grineer, Str 1.0:
  `WeaponHit = 100 × 2.65 × 2.0 × 1.3 = 689`; `XH = 689 × 0.26 × 1.3 = 232.9` Void before armor. On a 2× head: both numbers double (`WeaponHit` 1378, XH 465.8 because Xata keeps the body-part multiplier).
- The Vast Untime (Xaku 4) gives +50 % Void vulnerability on affected enemies (§2).

### 1.4 Saryn – Toxic Lash (inspiration: yes)

- Value: `20 / 24 / 26 / 30 %` `× Str` of each hit as an **Extra Hit** of Toxin; **doubled for melee** (`40 / 48 / 52 / 60 %`). Guaranteed Toxin proc on every extra hit. 45 s. Not subsumable (Saryn is not on the Helminth Toxic Lash list; `Ability/data` has no `Subsumable`).
- `XH = WeaponHit × p × (1 + ΣFACTION)` with the forced **1× weak-point multiplier** (`Extra Hit` § Notes; contradicted by `Sonar` § info, which says extra hits get the weak-spot bonus again → Open question Q3).
- DoT of the forced Toxin proc: the extra hit is the DoT seed, and **all** weapon Toxin mods (even when combined into Viral/Corrosive/Gas) multiply the tick again; faction applies a third time; Elementalist mods and the Emerald shard add to status damage. Saryn passive adds +25 % status duration.
  `tick = XH × (1 + ΣToxinMods) × 0.5 × (1 + ΣFACTION) × (1 + ΣStatusDamage)` (DoT coefficient 0.5 per `Damage/Toxin Damage`).
- Worked example (`Extra Hit` notes): 100 base, Infected Clip +90 %: `XH = 100 × 1.9 × 0.3 = 57`; Toxin tick `57 × 1.9 × 0.5 = 54.15` per tick.
- Worked example (`Toxic Lash`): unmodded 200-damage hit, rank 3, Intensify: `XH = 200 × 0.3 × 1.3 = 78`; tick `78 × 0.5 = 39`.
- Interactions: does not combine with weapon elements; does not charge Incarnon; Nourish's Viral does **not** count as a matching Toxin bonus for the tick (`Extra Hit` § Notes); a Spore-afflicted enemy takes two extra hits but one Toxin stack.

### 1.5 Grendel – Nourish (inspiration: yes)

- Value: Viral weapon damage `+40 / +50 / +60 / +75 %` `× Str`. Subsumed: `+24 / +30 / +36 / +45 %` `× Str` since **U35.5** (was 75 %). Buff radius 25 m, 25 s. Source: `Nourish` § info and § helminth; `Helminth` patch U35.5.
- Bucket: **ELEM**, as a Viral bonus "additive with existing elemental and IPS mods; think of it like adding a +75 % Viral mod". It is an already-combined element, so it does not combine further and does not count as Toxin/Cold for DoT scaling. It is a Universal Weapon Bonus (also hits weapon-coded abilities).
- DoT: Viral is not a DoT. It does add Viral weight to the status table (more Viral procs).
- Worked example: 100 base, Serration, Infected Clip (+90 % Toxin), Nourish at Str 1.3 (`0.975`): total `100 × 2.65 × (1 + 0.9 + 0.975) = 761.9`, of which Viral `100 × 2.65 × 0.975 = 258.4` and Toxin `238.5`.

### 1.6 Chroma – Vex Armor (Fury) (inspiration: no)

- Value: up to `200 / 225 / 250 / 275 %` `× Str` (Fury). Builds `2.75 %/HP` lost or `15 %` per ranged kill (×2 on weak-point kills); 100 HP lost or 19 kills to cap. Range 18 m, 25 s. Not subsumable. Multiple Chromas' Fury stacks.
- Bucket: **BASE** ("additive multipliers that function the same as … Serration"). Also on Chroma's abilities. "Applies once additively with Pressure Point effects and once multiplicatively on Exodia Contagion and glaive explosions."
- Worked example (`Vex Armor`): `100 × (1 + 1.65 + 2.75 × 1.3) = 622.5`.
- Recast with higher Strength raises the cap only if the buff is not already capped.

### 1.7 Chroma – Elemental Ward, Toxin (inspiration: no)

- Value: on holster, `15 / 25 / 30 / 35 %` `× Str` weapon damage for 3 s; also `+35 %` reload speed scaled by **Duration**. Subsumable (element follows the user's primary energy colour).
- Bucket: **BASE**, "additive with Vex Armor's Fury and mods like Serration"; Universal bonus (same class as Vex Armor and Arcane Arachne). Elemental Wards from several Chromas do not stack.

### 1.8 Harrow – Covenant (inspiration: yes, as generic "flat crit chance")

- Value: Retaliation crit chance `5 % + 1.5 % per 100 damage absorbed`, both `× Str`; ×4 on weak-point hits; cap **50 %** body, **200 %** weak point. Lasts `12 s × Duration`. Applies to Harrow, allies in Affinity Range and companions. Not subsumable.
- Bucket: **CC_abs** ("flat value applied after mods").
- Worked example (`Covenant`): 10 % base CC, Point Strike +150 %, capped buff, weak-point hit: `10 % × 2.5 + 200 % = 225 %`. With Arcane Avenger also: `+245 %` on weak points, `+95 %` on body (`Covenant` § tips).

### 1.9 Kullervo – Wrathful Advance (inspiration: yes, as "flat crit chance (~Kullervo, Harrow)")

- Value: `+50 / +100 / +150 / +200 %` **final** melee crit chance `× Str`, 10 s × Duration after cast. Subsumed: `+25 / +50 / +75 / +100 %` `× Str` (also applies to exalted melee). Source: `Wrathful Advance`.
- Bucket: **CC_abs**, melee only. Example: 25 % melee → 225 % at rank 3, Str 1.0.
- Kullervo passive: +75 % heavy-attack efficiency, +100 % heavy wind-up speed (no damage).

### 1.10 Octavia – Amp and Metronome (inspiration: no)

- Amp: `+25 %` (min) to `+200 %` (max) weapon and melee damage `× Str`, depending on environmental sound; 14 m field, 30 s. Bucket **BASE** (additive with Serration and with Pressure Point). Overlapping Amps: only the highest applies. Double-dips on Exodia Contagion (`wiki-additive-vs-multiplicative-edge-cases.md` §4). How sound level maps to the value: not documented → UNVERIFIED; a calculator should take the % as input.
- Metronome Forte: `+30 %` melee damage `× Str` (BASE, additive with Pressure Point). Metronome Opera: `+30 %` ranged **multishot** `× Str` (additive with Split Chamber). Source: `Metronome`.

### 1.11 Wisp – Haste Mote; Breach Surge (inspiration: no)

- Haste Mote (Reservoirs): `+30 %` fire rate, `+20 %` melee attack speed, `× Str` (`Reservoirs` § Haste). Additivity with fire-rate mods is not stated → UNVERIFIED (assume additive like every other fire-rate buff).
- Breach Surge: sparks are a weapon-coded ability, not a weapon buff (spark = 2× Str the proccing hit, 100 % crit, 1.5×). Not a weapon damage buff.

### 1.12 Volt – Electric Shield; other buffing fields (inspiration: no)

- Electric Shield: shots through the shield get `+50 %` Electricity (**not** scaled by Strength) and `+100 %` crit damage. Source: `Electric_Shield`; `Buffing Fields`.
  - Electricity: ELEM-like bonus on base damage, **does not combine** with other elements, stacks additively per shield passed (3 shields → +150 %), and does **not** raise Electricity status damage.
  - Crit damage: **CD_mult**, "multiplicative to all other sources of critical damage, effectively doubling the damage of critical hits"; does not stack with more shields. Exact placement for orange/red crits (`2 × CritTier` vs `1 + tier × (2·CDM − 1)`) → UNVERIFIED.
  - Melee does not benefit (projectile melee attacks do); most abilities do not; AoE needs the main projectile to pass.
- Gyre – Conductive Sphere (augment): `+75 %` Electricity per field `× Str` (additive, uncombined), `+75 % × Str` Electricity status damage, `+1` forced Electricity proc per field.
- Odonata – Energy Shell: `+50 %` Heat (uncombined), `+100 %` crit damage (multiplicative).
- Nyx – Assimilate: `+100 %` crit damage (multiplicative), not for Nyx's own projectiles.
- Weapon-made fields (Mutalist Quanta alt-fire, Simulor orbs, Mandonel charged shot) also buff allies' projectiles (−34 % total damage per field plus Electricity, flat crit, etc.); see `Buffing Fields`. A projectile can only take benefits from one source of field ("prevents the projectile from receiving benefits of buffing fields from other sources", `Buffing Fields` intro).

### 1.13 Gauss – Redline, Kinetic Plating (inspiration: no)

- Redline: fire rate `+15–75 %`, melee attack speed `+8–40 %`, reload `+10–50 %`, scaled by **Ability Duration** and by battery level. Exact battery → value curve: not documented → UNVERIFIED.
- Kinetic Plating + Redline above 80 % battery: `+100 %` melee damage (additive with Pressure Point), 100 % follow-through. Not strength-scaled.

### 1.14 Garuda passive (inspiration: no)

- `+5 %` weapon and melee damage per kill/assist, max `+100 %`; decays 1 %/2 s after 5 s without a kill. **MULT**, multiplicative with Eclipse, Overdriver, faction and Roar; applies twice on Exodia Contagion; omitted from Adding CO. Source: `Garuda/Abilities/Passive`; `Universal Weapon Bonuses`.

### 1.15 Nidus – Parasitic Link (inspiration: no)

- Linked to an ally: both get `10 / 15 / 20 / 25 %` weapon damage `× Str` and the same % as a **multiplicative** Strength bonus. Source: `Parasitic_Link`.
- Damage bucket: **MULT** ("multiplicative to other damage bonus sources such as Serration"); does **not** multiply Adding CO. Example on the page (200 % Str → +50 %): `100 × 1.5 + 100 × 0.8 = 230` with one-status CO, not `100 × 1.5 × 1.8 = 270`.
- Strength bonus: `bonus = 0.25 × Str_Nidus`; then `Str_new = Str_own × (1 + bonus)`. Example: Nidus at 145 % → bonus 36.25 % → Nidus `1.45 × 1.3625 = 197.6 %`. The linked ally multiplies **their own** Strength by the same 1.3625.

### 1.16 Banshee – Sonar (inspiration: no; a "headshot / weak point multiplier" input could approximate it)

- Highlighted body part: damage `× 2 / 3 / 4 / 5` `× Str` (rank 3 = 5×; Intensify 6.5×). 30 s. Not subsumable. Source: `Sonar`.
- Multiplies on top of the natural body-part multiplier (multiplicative); several highlighted nodes on one body part multiply with each other. Sonar spots count as weak points for weak-point effects (Acuity, Incarnon, Worthy Comradery, Covenant) but do **not** become head-crits and do not give the enemy's natural head multiplier to other parts.
- DoT: procs from a Sonar hit inherit the boosted hit; AoE DoTs (Electricity, Gas, Blast 10-stack) can double-dip if they also hit a Sonar spot.
- Example: Str 1.3, a headshot (2× on a normal enemy) that is also Sonar-highlighted: `2 × 6.5 = 13×`.

### 1.17 Equinox – Metamorphosis (Day) (inspiration: no)

- `10 / 15 / 20 / 25 %` weapon damage `× Str` right after switching to Day form, **decaying linearly to 0** over `25 s × Duration`. **BASE** (additive with Serration). Not for melee. Source: `Metamorphosis`. Time-average over its duration = half the peak.

### 1.18 Mesa – Shooting Gallery; Jade – Symphony of Mercy; Nyx – Absorb; Limbo – Rift Torrent

- Shooting Gallery: `10 / 15 / 20 / 25 %` weapon damage `× Str`, ranged **and** melee, **BASE**. Subsumable.
- Symphony of Mercy, Deathbringer song: `40 / 60 / 80 / 100 %` weapon damage `× Str`, **BASE**; Power of the Seven song: `+25 %` Ability Strength `× Str`.
- Absorb (Nyx): weapon damage bonus after release, 8 s, cap `+400 %`: `bonus = sqrt(0.025 % × Str × AbsorbedDamage)` (`Absorb` § info). The unit of the result is ambiguous: `sqrt(0.00025 × 1.3 × 20000) = 2.55`, read as +255 % → UNVERIFIED. Bucket: listed on `Universal Weapon Bonuses` § Damage (base bucket by that page's convention) → bucket UNVERIFIED.
- Rift Torrent (Limbo augment): `+30 %` per enemy in Rift Surge `× Str`, base bucket; double-dips on Exodia.

### 1.19 Vauban passive and Overdriver; Ivara Navigator

- Vauban passive: `+25 %` damage vs incapacitated enemies, **MULT** on total damage, applied again to DoT if the proc was applied while the enemy was incapacitated (so it behaves like faction for DoT). Source: `Vauban/Abilities/Passive` example: `0.35 × 100 × 2.65 × 1.3 × 1.3 × 1.25 × 1.25 …`.
- Overdriver (Minelayer): `+25 %` `× Str`, **MULT**; omitted from Adding CO.
- Navigator (Ivara): projectile damage multiplier growing to `× 5` `× Str` (`× Duration` growth rate); projectiles only, not hitscan. **MULT**.

### 1.20 Valkyr – Warcry, Rage passive; Excalibur passive

- Warcry: `+15 / 20 / 25 / 50 %` melee attack speed `× Str` (additive with Fury); subsumed `9 / 12 / 15 / 30 %` `× Str`. Source: `Warcry`.
- Valkyr Rage (passive): up to `+300 %` melee damage, "additive with Pressure Point and Condition Overload" (`Valkyr/Abilities/Passive`).
- Excalibur Swordsmanship (passive): `+10 %` melee damage and `+10 %` attack speed with swords, dual swords, nikanas, rapiers and Exalted Blade; additive with Pressure Point / Fury.

### 1.21 Dagath – Grave Spirit; Citrine – Prismatic Gem

- Grave Spirit: `+35 / 40 / 45 / 50 %` crit damage on all her weapons `× Str` (**CD_rel**, additive with Vital Sense); `+70–100 %` `× Str` against Doomed enemies. Self only. Source: `Grave_Spirit`.
- Prismatic Gem: allies inside the aurora get `+60 / 70 / 85 / 100 %` status chance `× Str` (additive with Rifle Aptitude — i.e. relative status bucket) and `+100 %` status duration `× Duration`. Multiple Citrines stack. Beams: 4 hits of 1000 `× Str` Heat/Cold/Electricity/Toxin per ally per 0.4 s (ability damage, not a weapon buff).

### 1.22 Other element-adding buffs

| Buff | Value | Bucket / notes | Source |
|---|---|---|---|
| Lavos – Valence Formation (augment) | `+200 %` of the cast ability's element, 20 s | ELEM, **does not combine**; additive with matching element mods; adds `+200 %` to that element's DoT (Heat/Electricity/Toxin/Gas; not Blast) on all procs; multiplicative with faction and Elementalist; guaranteed proc; not removed by nullifiers | `Valence_Formation` |
| Hydroid – Plunder | Corrosive `+10 %` per enemy and per consumed Corrosive stack, cap `+200 %`, `× Str` (Universal page: "up to 200 %") | ELEM (Corrosive); also on Hydroid's abilities. Combine behaviour UNVERIFIED | `Plunder`; `Universal Weapon Bonuses` |
| Cyte-09 – Resupply | Extra Hit `10 / 15 / 20 / 25 %` `× Str` of chosen element, **doubled for snipers**, guaranteed proc; not on battery weapons, not on Incarnon forms | XH | `Extra Hit` table |
| Oraxia – Silken Stride | Extra Hit `10–40 %` `× Str` Toxin, doubled while wall-latching, guaranteed proc | XH | `Extra Hit` table |
| Uriel – Demonium rune (passive) | Extra Hit `30 %` `× Str` Heat, guaranteed proc | XH | `Extra Hit` table |
| Narin – Sangodae (passive) | Extra Hit `50 %` Cold, guaranteed proc; also on her abilities | XH | `Extra Hit` table |
| Gastroparesis (augment) | Extra Hit `15 %` Gas per enemy consumed, guaranteed proc | XH | `Extra Hit` table |

### 1.23 Helminth summary (values at 100 % Str)

| Ability | Full | Subsumed | Changed | Source |
|---|---|---|---|---|
| Roar | +50 % faction | +30 % | Helminth introduction (U29, date UNVERIFIED) | `Roar` § helminth |
| Eclipse (Solar) | +200 % MULT | +30 % (DR cap 75 %) | U35.5 (was 150 %) | `Helminth` patch U35.5 |
| Xata's Whisper | 26 % XH | 26 % XH (no reduction listed) | — | `Xata's Whisper` § helminth |
| Nourish | +75 % Viral | +45 % Viral (energy ×1.6, explosion 1 stack, no heal) | U35.5 (was 75 %) | `Nourish` § helminth; `Helminth` U35.5 |
| Wrathful Advance | +200 % CC_abs melee | +100 % (also exalted melee) | — | `Wrathful_Advance` |
| Warcry | +50 % attack speed | +30 % | — | `Warcry` |
| Elemental Ward | as Chroma | same (element = primary energy colour) | — | `Elemental_Ward` |
| Shooting Gallery, Breach Surge, Terrify, Pillage | as original | not reduced on their pages | — | pages |

Restriction: Eclipse, Roar and Xata's Whisper can only replace Vex Armor (Chroma), Resupply (Cyte-09), Eclipse (Mirage), Amp (Octavia), Silken Stride (Oraxia), Roar (Rhino), Ripper's Wail (Temple), Demonium (Uriel), Xata's Whisper (Xaku) on those frames, so at most one of {that frame's own damage buff, subsumed Roar/Eclipse/Xata} is possible on them (`Helminth` § Damage buff restrictions). A build optimizer must enforce this when it chooses a self-cast subsumed buff.

---

## 2. Damage vulnerability on the target (external debuffs)

`Damage_Vulnerability` § Mechanics: vulnerability multiplies damage taken, **multiplicatively** with damage bonuses and with other vulnerabilities, except the listed additive group; because it sits on the target it does **not** double-dip DoT (the tick is multiplied once when it lands). Since U37 several of them also apply to shields and Overguard.

| Source | Value | Notes | Source page |
|---|---|---|---|
| Nova – Molecular Prime | +100 % (fixed) | also shields/OG since U37 | `Molecular_Prime` |
| Nezha – Blazing Chakram | +60 / 75 / 85 / 100 % `× Str` | | `Blazing_Chakram` |
| Khora – Strangledome | +100 % (page infobox misc says 200 % for non-Whipclaw → conflict, Q8) | not for Whipclaw | `Strangledome` |
| Mag – Magnetize | ×1.25 / 1.5 / 1.75 / 2 `× Str` (main target only) | | `Magnetize` |
| Atlas – Petrify | +50 % (fixed) | additive group | `Petrify` |
| Equinox – Rage | +20 / 30 / 40 / 50 % `× Str` | | `Rest_&_Rage` |
| Titania – Razorwing drones | +100 % (fixed) | | `Razorwing` |
| Valkyr – Paralysis | +20 / 30 / 40 / 50 % `× Str`, **melee only** | | `Paralysis` |
| Gara – Mass Vitrify | +20 / 30 / 40 / 50 % `× Str` | | `Mass_Vitrify` |
| Qorvex – Containment Wall | +25 % | additive group | `Containment_Wall` |
| Sevagoth – Reap | +10 / 20 / 30 / 50 % `× Str` | no refresh until expired | `Reap` |
| Xaku – The Vast Untime | +50 % **Void only** | boosts Xata's Whisper | `Xata's Whisper` |
| Jade – Judgement (passive) | additive group (value not read) | UNVERIFIED value | `Damage_Vulnerability` § Notes |
| Prey of Dynar, Atomi-Barrage | additive group | | idem |
| Magus Accelerant / Magus Destruct (operator arcanes) | +65 % per application, **Heat** / **Puncture** only, permanent, stacks additively without limit | example: Ignis Wraith `35 × 2.65 × 1.9 × (1 + 3 × 0.65)` | `Magus_Accelerant` |
| Theorem Contagion | +200 % to the orb's damage type, 6 s | additive group | `Module:Arcane/data` |
| Mecha Empowered (Kubrow mark, set) | +150 % per aura, "applied each time damage is calculated" → **twice on DoT** (6.25× with one aura) | behaves like faction, not like vulnerability | `Mecha_Empowered` |
| Viral status (external) | `×(2 + 0.25(n−1))` on health, max ×4.25 at 10 stacks | see `wiki-damage-types-and-status-effects.md` | — |

Formula: `received = hit × Π(1 + V_mult) × (1 + ΣV_additive_group) × (Viral factor on health)`.

---

## 3. Armor strip (external) and external Viral

Detailed formula is in `wiki-enemy-defenses-and-scaling.md` §3.3; the external-source parts:

```
Armor_after = Armor_clamped
            × max(0, 1 − Σ(ability strips of TOTAL armor) − 0.18 × N_CorrosiveProjection × AuraStrength)
            × (1 − HeatStrip(t))                  # 0.5 at full Heat
            × (1 − min(1, 0.20 + 0.06 × CorrosiveStacks))   # 0 stacks → 1
```

- Corrosive Projection: `−3 %` per rank, `−18 %` at rank 5 (since U27.2), additive with other Corrosive Projections (4 × 18 % = 72 %) and with ability strips; multiplicative with Heat and Corrosive status (`Corrosive_Projection`). Coaction Drift raises the wearer's aura by ×1.15 (aura strength; Corrosive Projection is a group aura so only Aura Strength applies, `Coaction_Drift`).
- Ability strips (all `× Str`, of total armor, additive with each other): Terrify 60 % (temporary), Fusion Strike 50 %, Sonic Boom 70 %, Avalanche 60 %, Pulverize 50 %, Regurgitate 75 %, Pillage 25 % (shields first), Fracturing Crush 75 %, Psychic Bolts 80 % (temporary), Reckoning 60 %, Tharros Strike 50 %, Abating Link 60 % (temporary), Seeking Shuriken 70 % (8 s), Plein Air 50 %, Rakhali's Cavalry 35 %/hit, Gaze 50 % (aura), Bastille 10 %/s, Ophanim Eyes 10 %/s, Temporal Erosion 10 %/hit, Fire Blast 50–100 %, Smite (all), Thermal Sunder Blast up to 100 % by battery. Unairu Caustic Strike (operator): 100 % at max rank. Values from `Armor` § Removing Enemy Armor.
- Status strips from other players count (Heat full = 50 %, Corrosive 20 % + 6 %/stack, cap 10 stacks; Emerald shard +2/+3 max stacks; Hydroid passive first stack 50 %).
- Worked example: 1 Corrosive Projection + Terrify at 100 % Str: `1 − 0.18 − 0.60 = 0.22` of armor left. Add full Heat and 10 Corrosive stacks with four Corrosive Projections: `(1 − 0.72) × 0.5 × (1 − 0.8) = 0.028` (2.8 % left).
- External Viral (inspiration's "External virus"): an ally or companion keeps N Viral stacks on the target; the health multiplier is `2 + 0.25 (N − 1)` (N ≥ 1). It also counts as one more status type for Condition Overload-type mods (inspiration JS L12015; game: CO counts status **types** on the enemy regardless of who applied them — `Condition Overload (Mechanic)`; consistent).

---

## 4. Auras (max rank)

| Aura | Value | Bucket | Stacks across squad | Source |
|---|---|---|---|---|
| Steel Charge | +60 % melee damage | BASE (additive with Pressure Point); affects melee exalted, thrown glaives, Archmelee, Blade Storm | yes, additive (4 × 60 %) | `Steel_Charge` |
| Rifle Amp | +27 % rifle damage (rifles, bows, snipers, launchers; not shotguns) | BASE | yes (108 %) | `Rifle_Amp` |
| Pistol Amp | +27 % secondary damage (also Glory, Regulators, Dex Pixia, Balefire) | BASE | yes | `Pistol_Amp` |
| Shotgun Amp | +18 % shotgun damage | BASE | yes | `Shotgun_Amp` |
| Dead Eye | +52.5 % sniper-rifle damage (not bows/launchers) | BASE | yes | `Dead_Eye` |
| Holster Amp | +60 % weapon damage for 3 s after holstering | BASE (additive with Vigorous Swap, Arachne) | yes, "per squadmate with the mod" | `Holster_Amp`; `Universal Weapon Bonuses` |
| Worthy Comradery | +75 % crit chance on weak points | CC_rel (additive with Point Strike, Acuity); applies on Sonar spots | per squadmate | `Worthy_Comradery` (U38) |
| Corrosive Projection | −18 % enemy armor | §3 | additive | `Corrosive_Projection` |
| Growing Power | +25 % Ability Strength for 6 s after a weapon status proc | strength (additive) | bonus stacks, duration does not | `Growing_Power` |
| Power Donation | −30 % own Strength, +30 % allies' Strength | strength (additive) | four → +60 % each | `Power_Donation` |
| Empowered Blades | +60 % status chance and +60 % status damage on heavy attacks; drains 90 shields per hit | relative status chance; status damage (additive with Elementalist) | | `Empowered_Blades` |
| Mecha Empowered | +150 % vs Kubrow/Predasite-marked enemies | separate multiplier, twice on DoT (§2) | per aura | `Mecha_Empowered` |
| Swift Momentum | +30 % heavy wind-up speed, +6 s combo duration | — | | `Swift_Momentum` |
| Ready Steel | +24 initial combo | combo | | `Ready_Steel` |
| Shield Disruption / Infested Impedance | −18 % enemy shields / slow | not damage | | |

Coaction Drift (exilus): `+15 %` aura strength (your aura for everyone) and `+15 %` aura effectiveness (everyone's auras on you), so your own aura on yourself is `×1.15² = 1.3225` (Steel Charge → +79.35 %), allies get `×1.15`. Fixed in U43 for Power Donation.

---

## 5. Archon shards (Helminth; 5 slots; Tauforged = 1.5×)

Source: `Archon Shard` § Shard Buffs and the per-colour pages. Every bonus below stacks with the same bonus on other shards (multiple shards of one colour can pick the same bonus).

| Colour | Bonus that changes weapon damage | Normal (Tauforged) | Bucket |
|---|---|---|---|
| Crimson | Melee crit damage | +25 % (+37.5 %) | CD_rel, additive with Organ Shatter; affects exalted melee |
| Crimson | Primary status chance | +25 % (+37.5 %) | relative, additive with Rifle Aptitude; exalted primaries too |
| Crimson | Secondary crit chance | +25 % (+37.5 %) | CC_rel, additive with Pistol Gambit; exalted secondaries too |
| Crimson | Ability Strength | +10 % (+15 %) | strength, additive (scales every strength-scaled self-cast buff) |
| Crimson | Ability Duration | +10 % (+15 %) | duration (Redline, buff uptime) |
| Violet | Primary Electricity | `+30 % (+45 %)` plus `+10 % (+15 %)` per Crimson, Azure **or Violet** shard equipped, the shard itself included | ELEM, additive with Stormbringer, **combines** with mods |
| Violet | Melee crit damage | +25 % (+37.5 %); doubled when max Energy > 500 (501+) | CD_rel |
| Violet | Ability damage vs Electricity-afflicted | +10 % (+15 %) | ability damage only, not exalted weapons |
| Topaz | Secondary crit chance per kill on Heat-afflicted enemy | +1 % (+1.5 %) per kill, max +50 % (+75 %) per shard | CC_rel, additive with Pistol Gambit; resets on revive |
| Topaz | Ability damage vs Radiation-afflicted | +10 % (+15 %) | ability damage only |
| Emerald | Toxin status damage | +30 % (+45 %) | status damage, additive with Elementalist; weapon **and** ability Toxin procs (incl. Toxic Lash) |
| Emerald | Max Corrosion stacks | +2 (+3) | armor strip (§3); also raises capped bosses (4 → 6/7) |
| Emerald | Ability damage vs Corrosion-afflicted | +10 % (+15 %) | ability damage only |
| Azure / Amber | none on weapon damage (health, shields, armor, energy, casting speed, orbs) | | |

- Violet Electricity worked examples (`Violet_Archon_Shard` § Tips, Tauforged): 1 shard alone `0.45 + 0.15 = 60 %`; 2 Violet `2 × (0.45 + 0.15 × 2) = 150 %`; 2 Violet + 3 Crimson `2 × (0.45 + 0.15 × 5) = 240 %`; 5 Violet `5 × (0.45 + 0.75) = 600 %`. Normal: 1 shard alone 40 %, 5 Violet `5 × (0.3 + 0.5) = 400 %`.
- Violet element merge order: weapon's modded elements combine first; then innate element, Kuva/Tenet progenitor element and the Violet Electricity are added in HCET order (Heat → Cold → Electricity → Toxin). Example: Tenet Glaxion (innate Cold) + Heat progenitor + modded Toxin + Violet Electricity → Toxin+Heat = Gas, then Cold+Electricity = Magnetic (`Violet_Archon_Shard`).
- Crimson secondary crit example: 20 % base, Pistol Gambit +120 %, Tauforged Crimson: `0.20 × (1 + 1.2 + 0.375) = 51.5 %`.
- Topaz example: 5 Tauforged Topaz, after 50 qualifying kills: `+375 %` secondary CC_rel.
- Ability damage (Violet/Topaz/Emerald): "unique modifier … multiplicative with Ability Strength scaling, does not apply to Exalted Weapons", bonuses of different colours add (2 Violet + 1 Topaz vs Electricity + Radiation target = +30 %); the status must already be on the enemy before the hit.
- Changes: introduced U32 (Crimson/Amber/Azure); Violet/Topaz/Emerald U35; Violet melee-crit double-application bug fixed U35.0.9 (patch text says "+37 %" for Tauforged, the table says 37.5 %, Q6); Crimson primary status / secondary crit restricted to their slot in U32.1.4; Arcane Battery counts toward the 500-energy threshold since U42.

---

## 6. Focus schools (Operator; buffs that reach Warframe weapons)

| School / node | Value (max) | Bucket | Source |
|---|---|---|---|
| Madurai – Phoenix Talons (passive) | +30 % "Physical Damage" for Warframe weapons (not abilities; yes exalted weapons and Necramechs); +30 % operator damage | special, see below | `Focus/Madurai` |
| Madurai – Void Strike | First operator ability consumes all energy: `+10 %` damage per 1 % energy consumed → `+1000 %` at full, for 8 s, 40 s cooldown; for Warframe weapons **additive with Serration/CO/Secondary Merciless**; not for abilities or archguns | BASE | `Focus/Madurai` |
| Madurai – Sling Strength | +40 % Ability Strength for 20 s after a chained Void Sling | strength (additive) | idem |
| Madurai – Overwhelming Power (Tauron) | +5 % Strength per enemy hit by Tauron Strike, max +50 %, 30 s | strength | idem (U41) |
| Naramon – Lethal Levitation | +50 % weapon damage per Lifted enemy attacked by the Operator, 60 s, max 4 stacks → +200 %; melee included since a later fix | BASE (additive with Serration) | `Focus/Naramon` |
| Naramon – Killer's Rush | +50 % melee crit chance for 40 s after an operator finisher | CC_rel (additive with True Steel: `5 % × (1 + 1.2 + 0.5) = 13.5 %`) | idem |
| Zenurik – Temporal Shot | +100 % precision headshot damage on Temporal-Drag-slowed enemies | headshot bucket (exact bucket UNVERIFIED) | `Focus/Zenurik` |
| Zenurik – Hardened Wellspring | +20 % Ability Strength inside a reinforced Wellspring | strength | idem |
| Zenurik – Spectral Pages (Tauron) | +10 % Strength per page (up to 4), ×1.40 with Boundless Energy | strength | idem |
| Unairu – Caustic Strike | Operator bomb strips 100 % armor (8 m) | §3 | `Focus/Unairu` |
| Vazarin – Eroding Rapids (Tauron) | +100 % status vulnerability in the vortex | target status vulnerability (bucket UNVERIFIED) | `Focus/Vazarin` |

Phoenix Talons formula (`Focus/Madurai` § Phoenix Talons): the bonus is `0.3 × TotalBaseDamage` (physical **and** elemental base included), split over the physical types by their share of the **physical** base, and "added as a flat bonus after physical mods". Worked example on the page, Dual Ichor (19 I, 11 P, 45 S, 47 Toxin; total 122):
`Impact 19 + 122 × 0.3 × 0.254 = 28.30`, `Puncture 11 + 122 × 0.3 × 0.146 = 16.34`, `Slash 45 + 122 × 0.3 × 0.6 = 66.96`, `Toxin 47`. Total 158.6 (+30 % of total). The inspiration instead multiplies physical types by 1.3 (`Impact 24.7, Puncture 14.3, Slash 58.5, Toxin 47`, total 144.5), so it under-counts weapons with innate elements and has no added physical for purely elemental weapons. Whether the flat bonus is multiplied by Serration-type BASE mods (most likely, since it is part of base damage) is not stated → UNVERIFIED. Void damage gets nothing from Phoenix Talons (`Damage/Void Damage`).
Madurai 3.0 dates from U31.5; it no longer gives elemental damage. Tauron nodes U41.

---

## 7. Warframe arcanes that buff weapons (max rank)

Values from `Module:Arcane/data` (2026-11-21) and the arcane pages. Proc-chance arcanes need an uptime model (inspiration treats them as always on when installed).

| Arcane | Trigger / value | Bucket | Source |
|---|---|---|---|
| Arcane Avenger | on damaged 21 %: +45 % crit chance, 12 s, all weapons and some abilities | CC_abs (`25 % × 2.2 + 60 % + 45 %` with True Steel + Cat's Eye on Atterax = 160 %) | `Arcane_Avenger` |
| Arcane Fury | on crit 60 %: +180 % melee damage, 18 s | BASE (additive with Pressure Point) | `Arcane_Fury` |
| Arcane Strike | on hit 15 %: +60 % melee attack speed, 18 s; also exalted melee | attack speed (additive with Fury) | `Arcane_Strike` |
| Arcane Acceleration | on crit 30 %: +90 % fire rate, primaries except shotguns, 9 s; **not** doubled for bows | fire rate (additive) | `Arcane_Acceleration` |
| Arcane Velocity | on crit 90 %: +120 % pistol fire rate, 9 s | fire rate (additive) | `Arcane_Velocity` |
| Arcane Tempo | on crit 15 %: +90 % shotgun fire rate, 12 s | fire rate (additive) | `Arcane_Tempo` |
| Arcane Momentum | on crit 60 %: +150 % sniper reload, 12 s | reload | `Module:Arcane/data` |
| Arcane Rage | on headshot 15 %: +180 % primary damage, 24 s | BASE | `Arcane_Rage` |
| Arcane Rise | on reload 60 %: +150 % primary damage, 24 s | BASE (page empty; by analogy with Awakening → UNVERIFIED) | `Module:Arcane/data` |
| Arcane Awakening | on reload 60 %: +150 % pistol damage, 24 s | BASE | `Arcane_Awakening` |
| Arcane Precision | on weak-point hit (since U44; was headshot): +300 % secondary damage, 18 s, not refreshed by further hits | BASE | `Arcane_Precision` |
| Arcane Primary Charger | on melee kill 30 %: +300 % primary damage, 12 s | BASE (page example uses 200 %; current max is 300 %) | `Arcane_Primary_Charger` |
| Arcane Blade Charger | on primary kill 30 %: +300 % melee damage, 12 s | BASE | `Arcane_Blade_Charger` |
| Arcane Arachne | on wall latch: +150 % primary/secondary damage, 30 s, no refresh | BASE on guns; separate multiplier on some melee projectiles (see edge cases) | `Arcane_Arachne` |
| Arcane Pistoleer | 60 % on pistol weak-point kill: infinite ammo 12 s (U44.1.3) | no damage | other finding |
| Arcane Hot Shot | +6 % crit chance per Heat proc inflicted **by abilities**, 10 s, max 50 stacks = +300 %; all weapon types | CC_rel (additive with Point Strike) | `Arcane_Hot_Shot` |
| Arcane Crepuscular | while invisible: +30 % Strength and +3.0× final crit multiplier | strength; CD_abs `1 + tier × (CDM − 1 + 3)`; Skana 1.5× → 4.5 / 8 / 11.5× for yellow/orange/red | `Arcane_Crepuscular` |
| Molt Augmented | +0.24 % Strength per kill, 250 kills → +60 %, kept until death | strength (additive) | `Molt_Augmented` |
| Molt Vigor | +45 % Strength on next cast after an operator ability | strength | `Module:Arcane/data` |
| Arcane Bellicose | `round(MaxHealth / 250 × 6 %)`, max +72 % at 2980 HP | strength | `Arcane_Bellicose` |
| Arcane Power Ramp / Impetus / Ice Storm / Camisado | +9 %×4 / +6 % per unique ability status / +2 %×20 / up to +60 % | strength | `Module:Arcane/data` |
| Theorem Demulcent | in a Residual zone: +12 % weapon damage per second, 15 stacks = +180 %, persists 20 s | bucket UNVERIFIED (page empty) | `Module:Arcane/data` |

Arcane values were normalised in U31.6 ("only one stat is affected by rank"); the R5 numbers above are the post-U31.6 values.

---

## 8. Companion and squad buffs

| Source | Value | Bucket | Source page |
|---|---|---|---|
| Adarza Kavat – Cat's Eye | +60 % crit chance (absolute), 10 s, 20 s cooldown, 25 m | CC_abs | `Cat's Eye` |
| Smeeta Kavat – Charm (crit buff) | **sets** crit chance to 200 % if lower, 30 s, 10 % of Charm procs | floor: `CC = max(CC, 200 %)` | `Charm` (contradicts `Universal Weapon Bonuses`, which lists it as "+200 %" absolute, Q5) |
| Tenacious Bond (companion mod) | +1.2× final crit multiplier if companion crit chance ≥ 50 % | CD_abs | `Tenacious_Bond` |
| Vigorous Swap (Warframe mod) | +165 % for a few seconds after swap | BASE (additive with Holster Amp, Arachne) | `Vigorous_Swap` |
| Holster Amp / Rifle Amp / Steel Charge … | see §4 | | |
| True Master's Font – Damage Blessing (Relay) | +25 % | FACTION (additive with Bane and Roar) | `Universal Weapon Bonuses` § Faction |
| Crucible Overflow (Alchemy) | +100 % combined elements | ELEM | `Universal Weapon Bonuses` |
| Unairu Initiative (Perita Rebellion) | +100 % Impact per order on weapons (200 % on abilities), scales like an elemental (total base) | ELEM-like | `Universal Weapon Bonuses` |
| Vazarin Initiative (Perita Rebellion) | +25 % status chance per order | relative status | idem |
| Harmony heavy-kill | +30 % status duration 30 s to allies in affinity range | duration | idem |

---

## 9. Which buffs touch status DoT, and how often

| Buff kind | Direct hit | DoT tick (proc from that hit) |
|---|---|---|
| BASE (Vex Armor, Amp, Arcane Fury, Void Strike, auras…) | ×1 | ×1 (via modded base damage) |
| ELEM (Nourish, Violet, Electric Shield Electricity, Valence) | per type | Matching element bonus scales Heat/Toxin/Electricity/Gas ticks; **uncombined field Electricity (Electric Shield) does not**; Valence adds +200 % to its element's DoT; Nourish Viral has no DoT |
| MULT (Eclipse, Garuda, Parasitic Link, Overdriver, Navigator) | ×1 | ×1 (part of the hit) |
| FACTION (Roar, Bane, Damage Blessing) | ×1 | ×2 total (double dip); ×3 on Extra-Hit DoTs |
| Vauban passive, Mecha Empowered | ×1 | ×2 (if applied while condition holds) |
| Target VULN (Molecular Prime, Chakram…) | ×1 | ×1 (applied when the tick lands; no double dip) |
| Sonar | ×1 | ×1 via the boosted hit; AoE DoTs that also hit a Sonar spot ×2 |
| Crit (incl. CC_abs, CD_abs, CD_mult) | tier | tier of the inducing hit |
| Extra Hits (Toxic Lash, Resupply, Silken Stride…) | own instance | forced proc seeded by the extra hit; matching weapon element mods apply again |
| Status damage (Emerald, Empowered Blades, Conductive Sphere Electricity) | — | additive with Elementalist |

---

## 10. Inspiration coverage and what a thorough calculator should add

Inspiration exposes (per `inspiration-feature-inventory.md` lines 129-155, 263-269, 307-309):

| Inspiration option | Inspiration model | Game truth (this file) | Gap |
|---|---|---|---|
| Roar % (default 30) | added to faction sum | same (§1.1) | none; value entered as final %, not × Strength |
| Eclipse % | separate multiplier | same; but additive with Adding CO (§1.2) | check its CO handling |
| Xata's Whisper % | `round(sum(hit) × armor/DR × p)`, ×1.5 vs Overguard | Extra Hit with faction and body part re-applied (§1.3) | verify faction/head double-dip and Blast-detonation trigger |
| Toxic Lash % | extra toxin hit + toxin DoT | §1.4 | verify faction ×2/×3 and Toxin-mod re-application |
| Nourish % | adds Viral = base × p | §1.5 | ok |
| Flat crit chance (~Kullervo, Harrow) | absolute CC | Covenant has 50 %/200 % caps and ×4 weak point; Wrathful Advance melee-only | no weak-point split |
| Madurai | physical ×1.3, not exalted | Phoenix Talons adds 30 % of **total** base split over physical; applies to exalted weapons (§6) | formula differs |
| External Viral stacks | Viral stacks on target | §3 | ok |
| External armor strip % | % reduction of enemy armor | §3 (additive with CP, multiplicative with status) | fine as a single input; Steel Path bug noted in inventory |
| Ability strength | scales exalted weapons and strength-scaled mods only | also scales every self-cast buff | buffs are entered as final values |
| 12 auras (Ready Steel, Rifle/Pistol/Shotgun Amp, Worthy Comradery, Steel Charge, Dead Eye, Holster Amp, Swift Momentum, Corrosive Projection, Mecha Empowered, Empowered Blades), 17 Warframe arcanes, 27 Warframe mods, 18 shard options | | §4, §5, §7 | not present: Growing Power and Power Donation (only matter once Strength scales buffs), Coaction Drift (exilus), Crimson Strength / Topaz / Emerald ability-damage shard options, Rise-style arcanes are present |

Recommended additions (each with its bucket from this file): Vex Armor Fury (BASE), Octavia Amp (BASE) and Metronome Forte/Opera, Shooting Gallery (BASE), Deathbringer (BASE), Metamorphosis Day (BASE, decaying), Lethal Levitation and Void Strike (BASE, with uptime), Garuda passive / Parasitic Link / Overdriver / Navigator (MULT), Covenant with weak-point ×4 and caps, Wrathful Advance (melee CC_abs), Sonar (body-part multiplier), Electric Shield / Conductive Sphere / Energy Shell (field: uncombined element + crit-damage multiplier), Prismatic Gem (status chance), Grave Spirit (CD_rel), Haste Mote / Warcry / Redline (fire rate, attack speed), Smeeta Charm (CC floor 200 %), Cat's Eye (CC_abs), Tenacious Bond (CD_abs), Damage Blessing (FACTION), target vulnerabilities (Molecular Prime etc., with the additive group), Magus Accelerant/Destruct (type vulnerability), Mecha Empowered, Valence Formation, Plunder, Resupply / Silken Stride / Demonium extra hits, Crimson/Topaz/Emerald ability-damage and Crimson Strength shard options, Coaction Drift multiplier on auras, squad copies of the same aura, Helminth subsumed variants with the slot restriction.

---

## 11. Edge cases

1. Roar is FACTION: additive with Bane/Damage Blessing, multiplicative with everything else, double-dips DoT, triple-dips Extra-Hit DoT.
2. Eclipse is MULT except against Adding-type CO, where CO is added after Eclipse is applied (CO not multiplied).
3. Eclipse, Vex Armor, Arachne, Amp, Deathbringer, Rift Torrent and Garuda apply **twice** (or multiplicatively instead of additively) on Exodia Contagion and glaive explosions.
4. Parasitic Link damage is MULT but skips Adding CO; its Strength bonus is the only multiplicative Strength source.
5. Xata's Whisper keeps the body-part multiplier and re-applies faction; other extra hits use a forced 1× weak-point multiplier (but `Sonar` says extra hits get Sonar twice).
6. Xata's Whisper fires off Blast detonations (bug) with a third faction dip and elemental bonuses that Blast normally ignores.
7. Xata's Whisper and Toxic Lash are reduced by Ancient Disruptor / ability-damage reduction; other weapon buffs are not.
8. Toxic Lash melee value is doubled (60 %); Resupply doubles for snipers and skips battery weapons and Incarnon forms.
9. Toxic Lash ticks are re-multiplied by every Toxin mod on the weapon, even when that Toxin is combined into Viral/Corrosive/Gas; Nourish Viral does not count.
10. Extra hits trigger once per multishot pellet and on Acid Shells / Concealed Explosives, not on Bursting Mass or Pathocyst maggots; they do not trigger if the main hit kills.
11. Nourish Viral is additive in the elemental bucket and never combines; Electric Shield / Conductive Sphere / Energy Shell / Valence elements never combine; Violet shard Electricity **does** combine (HCET order with innate and progenitor elements).
12. Electric Shield Electricity does not raise Electricity DoT, while Conductive Sphere explicitly adds +75 % Electricity status damage and Valence adds +200 % DoT.
13. Electric Shield crit bonus does not stack across shields; its Electricity does (+50 % per shield).
14. Buffing fields: a projectile benefits from one field source only; melee swings and most abilities get nothing; glaive throws get nothing.
15. Covenant: 50 % body cap, 200 % weak-point cap, ×4 on weak points; Sonar spots count as weak points.
16. Wrathful Advance: melee only (and exalted melee for the subsumed version); counts down from cast.
17. Smeeta Charm sets CC to 200 % (floor), it does not add 200 %; Cat's Eye and Avenger add.
18. Sonar spots: multiplicative with natural body-part multiplier and with other Sonar nodes on the same part; no head-crit on non-head spots.
19. Metamorphosis Day damage decays linearly to 0 and does not affect melee.
20. Shooting Gallery affects melee too despite its name.
21. Equinox Rage, Molecular Prime etc. are target-side: multiplicative with all buffs, single application to DoT; Petrify/Containment Wall/Jade/Prey of Dynar/Magus/Theorem vulnerabilities add with each other.
22. Vauban passive and Mecha Empowered behave like faction for DoT (applied twice), unlike target vulnerabilities.
23. Phoenix Talons uses total base damage (elements included) but adds only physical; inspiration's ×1.3-physical differs for elemental-innate weapons.
24. Void Strike +1000 % is BASE (additive with Serration), so its real gain is `(1 + ΣBASE + 10)/(1 + ΣBASE)`, ≈ ×4.77 with Serration alone; 8 s on a 40 s cooldown.
25. Violet shard per-shard bonus counts the Violet shard itself; Tauforged per-shard bonus is 15 %.
26. Violet melee crit doubling needs max Energy **> 500** (501+); Arcane Battery counts since U42.
27. Archon ability-damage bonuses need the status on the enemy **before** the hit and never apply to exalted weapons.
28. Emerald Corrosion +stacks lets Corrosive reach 92 % (12 stacks) / 98 % (13) and raises 4-stack boss caps.
29. Helminth slot restriction: subsumed Roar/Eclipse/Xata cannot coexist with the host frame's own listed damage buff.
30. Ability Strength is the **caster's**: an ally's Roar/Eclipse value must not use the shooter's Strength. Power Donation lowers the caster's own Strength.
31. Coaction Drift: your own aura ×1.3225 on you, ×1.15 on allies; group auras (Corrosive Projection) get only ×1.15.
32. Arcane Acceleration is not doubled for bows; excludes shotguns. Arcane Precision is "on weak point" since U44 and does not refresh.
33. Arcane Arachne: BASE on guns, but on Exodia Contagion and thrown glaives it is a separate multiplier; additive on Exodia Epidemic, Tenet Agendus/Grigori, Wolf Sledge and some melee Incarnons; not on normal melee.
34. Arcane Crepuscular / Tenacious Bond are CD_abs: their relative gain falls as modded crit multiplier rises.
35. Absorb, Theorem Demulcent, Plunder and Haste Mote buckets are not documented (see Open questions).

---

## 12. Open questions

- Q1. Electric Shield / Energy Shell / Assimilate "+100 % crit damage, multiplicative": does it multiply the final crit tier multiplier or `CDM` before the tier formula? UNVERIFIED (matters for orange/red crits).
- Q2. Phoenix Talons: is the flat physical bonus multiplied by BASE mods (Serration)? Wiki says "added as a flat bonus after physical mods", silent on base mods. UNVERIFIED.
- Q3. Do non-Xata extra hits (Toxic Lash, Resupply) re-apply weak-point/Sonar multipliers? `Extra Hit` says no (forced 1×), `Sonar` says yes. Contradiction.
- Q4. Eclipse and other MULT buffs on Adding-CO weapons: are they also left out of the DoT seed? UNVERIFIED.
- Q5. Smeeta Charm: floor to 200 % (`Charm`, with a patch note about not lowering higher CC) vs "+200 % absolute" (`Universal Weapon Bonuses`). Prefer `Charm`; confirm.
- Q6. Tauforged Violet melee crit damage: 37 % (U35.0.9 patch text) or 37.5 % (shard table). Same question for Topaz-per-kill 1.5 %.
- Q7. Absorb weapon damage: result unit of `sqrt(0.025 % × Str × Dmg)` and its bucket.
- Q8. Strangledome vulnerability: +100 % (info text) vs 200 % (infobox misc).
- Q9. Bucket of Theorem Demulcent (+180 %), Arcane Rise, Plunder Corrosive (combines or not), Haste Mote / Redline fire rate (additive with mods?), Zenurik Temporal Shot (+100 % headshot: added to headshot multiplier or separate), Vazarin Eroding Rapids (status vulnerability definition).
- Q10. Octavia Amp: mapping from environmental sound to the 25–200 % value; no formula found.
- Q11. Redline battery-level → buff curve; documented only as ranges.
- Q12. Ability Strength floor (10 %?) and whether any buff caps at a Strength value (e.g. Covenant is capped by its own caps; Roar/Eclipse have no documented cap).
- Q13. Jade Judgement vulnerability value not read.
- Q14. Roar Helminth 30 % introduction date.

## 13. Data-source warning: `Module:Ability/data` stats are stale

The ability stats table in the wiki module (cached as `research/external/cache/poc/out/Ability_data_stats.json`) disagrees with the ability pages for exactly the buffs this file covers:
- Eclipse `HelminthValues = 150` (pre-U35.5; current 30) and "Damage Reduction Cap 95 %" (page: 90 %).
- Nourish lists "Toxin Damage On Digestion 2500", "Energy Multiplier 1.25x", "Strike Toxin Damage Multiplier 1.5x" (old Toxin-based Nourish; current: Viral +75 %, energy ×2).
- Terrify "Armor Reduction 20 %" (page and `Armor`: 60 %).
- Electric Shield "Critical Damage Bonus 200 %" (page: +100 % multiplicative; may be the same thing expressed as ×2).
Do not build buff values from that module; use the ability pages (or Public Export text) and these tables.
