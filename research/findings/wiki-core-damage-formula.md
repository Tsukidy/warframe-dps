Status: draft

# Core per-hit damage formula and order of operations (game-truth side)

Scope: how a weapon's listed base stats become the damage of one hit (one pellet / one damage instance) **before** enemy defenses (armor, faction resistances, shields, damage reduction). Status DoT is covered only where it interacts with the hit formula (faction double-dip, base for procs). Condition Overload, falloff, enemy scaling and armor are separate topics; they appear here only as buckets in the consolidated formula.

Research date: 2026-11-23. Sources are wiki.warframe.com wikitext (`?action=raw`) or rendered pages (`api.php?action=parse`) cached in `research/external/cache/wiki/`. "U" = game Update number as tagged by the wiki `{{ver|…}}` template. Release dates of updates are not given on the pages read, so most "last changed" entries only carry the update number.

Abbreviations: B = listed base damage per pellet (sum of all base damage types), CC = critical chance, CDM = critical damage multiplier, SC = status chance, MS = multishot, WP = weak point, HS = headshot.

---

## 1. Base damage and damage-type proportions

**Rule.** A weapon's base damage is a set of per-type values (Impact, Puncture, Slash, and/or elements). Base-damage bonuses scale every type by the same factor, so proportions are unchanged.

- Source: wiki `Calculating Bonuses` § "The Damage Application Order" step 1: "The added damage will be of the same damage type distribution the weapon innately deals." Karak example: 29 × (1 + 1.65) = 76.85.
- Source: wiki `Damage` § Physical Damage: "General damage increasing mods such as Serration affect all the damage types of a weapon."
- Physical damage cannot be added to a weapon that lacks that type (wiki `Damage` § Physical Damage; `Calculating Bonuses` § Calculating Physical Damage, Amprex + Fanged Fusillade = no effect). **Exception:** the Unairu Initiative buff (Perita Rebellion) adds Impact "like an elemental mod", i.e. as a percentage of total base damage, even on weapons with no Impact (wiki `Universal Weapon Bonuses` § Impact; UI shows 25%/order but it is 100%/order on weapons).

## 2. Base damage bonus bucket (Serration and friends)

**Formula:** `B_mod = B × (1 + Σ base_damage_bonuses)`

- All "+X% damage" sources stack additively in one bucket (internal operation type `STACKING_MULTIPLY`): wiki `Calculating Bonuses` § Overview table and § Additive Stacking (Serration 1.65 + Heavy Caliber 1.65 → 4.3×).
- Members confirmed by the wiki as additive with Serration-type mods: Vex Armor (`Calculating Bonuses` § Multiplicative Stacking note and § Opportunity Cost example), Primary Deadhead damage stacks (`Primary Deadhead` § Notes), Arcane Fury / Arcane Rage etc. (`Condition Overload (Mechanic)` § Math lists Serration, Vex Armor, Arcane Fury together as "Most Damage Bonuses"), Galvanized Shot/Aptitude/Savvy **when CO behaves additively** (see § 11), universal weapon-damage buffs (`Universal Weapon Bonuses` § Damage: Arcane Arachne, Holster Amp, Provoked, Vigorous Swap, Lethal Levitation, Vex Armor, Toxin Elemental Ward, Metamorphosis, Deathbringer, Rift Torrent, Shooting Gallery, Absorb, Amp). The page does not state explicitly that every universal buff is additive with Serration (UNVERIFIED for individual entries other than Vex Armor).
- Negative base damage mods (Hollow Point, Shrapnel Rounds, Frail Momentum, Vile Acceleration, Anemic Agility) are negative entries in the same bucket: `Calculating Bonuses` § Opportunity Cost item 5 (Anemic Agility −15% added into the sum).
- Serration is "applied before any other damage type", and the result is what elemental mods use: `Serration` § Notes (18 × 2.5 = 45).

Worked example: base 100, Serration (+1.65) + Primary Deadhead 3 stacks (+3.6) → 100 × (1 + 1.65 + 3.6) = 625.

## 3. Flat base-damage additions

### 3.1 Incarnon "Increase Base Damage by +X"
- Wording on Incarnon perks is "Increase Base Damage by +X" (wiki `Braton Incarnon Genesis` Daring Reverie / Evolution II perks; `Lex Incarnon Genesis` Hoplite Virtue: "+X", and "+80 for 8 s on Shield Break"). Patch history (`Incarnon` § Patch History) repeatedly calls these "Base Damage" (e.g. "Increased Base Damage from +50 to +75 (+100 to +150 for Vectis Prime)"; "Fixed the Lex Incarnon Evolution Hoplite Virtue not increasing the Lex's base damage").
- Evidence they are added **before** % base damage mods: wiki `Condition Overload (Mechanic)` § Math lists "Base Damage increases from Incarnon Genesis Evolutions" among effects the additive-CO recalculation **omits** (i.e. CO multiplies only the original base); `Braton Incarnon Genesis` § Notes: "Galvanized Aptitude's GunCO effect doesn't apply to the base damage bonuses from either Evolution II perk. Only the weapon's original base damage is multiplied." A forum user (forums.warframe.com topic 1357504) says flat Incarnon base damage "work[s] before your mods" (anecdotal).
- Proposed: `B' = B + X` (per pellet), then `B'` is multiplied by the base-damage bucket. **How X is split among damage types is not stated by any source read** → UNVERIFIED (proportional to existing types is the natural assumption, not confirmed).
- Some Incarnon "damage" bonuses are **not** base damage: Braton Incarnon Evolution II "Multishot consumes ammo … does +Y Damage" is "a unique modifier that is multiplicative to other damage buffs" and "only applies to shots generated by multishot" (`Braton Incarnon Genesis` § Notes lines 73-74).

### 3.2 Kuva / Tenet / Coda progenitor (Valence) bonus
- Bonus type ∈ {Impact, Heat, Cold, Electricity, Toxin, Magnetic, Radiation}; value 25 %–60 % "of the weapon's **base damage**"; "applies as weapon base damage, meaning elemental mods and status that scale from base / modified base damage will be affected" (wiki `Kuva Weapons` § Elemental Bonus; same text in `Tenet Weapons`, `Coda Weapons`).
- Rendered weapon pages: "which increases the listed base damage of the weapon by 25%-60%" and "This bonus stat is considered a base damage type and will be applied after all elemental mods for the purposes of elemental combinations. It uses the weapon's falloff range but always scales linearly from 100% to 0%, regardless of what the weapon's falloff damage scaling is." (`Kuva Drakgoon`, `Coda Hema`, `Tenet Detron` rendered pages, cached as `*.parsed.html`).
- Numbers check (wiki comparison lines "using max +60% Progenitor bonus"): Kuva Drakgoon charged 460 → 736 = 460 × 1.6; Tenet Detron 260 → 416; Coda Hema 52 → 83.2. So `bonus = valence% × B_listed`, added as a new base damage type.
- Valence Fusion: `final = min(1.1 × max(b1, b2), 60%)`, rounded **down** to 0.1 %, and ≥ 58 % rounds up to 60 % (wiki `Valence Fusion` § Adversary Weapons; 58 % rule added U36). Rolls uniformly distributed since U42 (`Kuva Weapons` ref).
- Worked example (Tenet Detron, per pellet): innate 26 Radiation, Electricity progenitor 60 % → +15.6 Electricity; base per pellet = 41.6. With Serration-type +100 %: 83.2 (both types doubled).

### 3.3 Other flat additions (not base damage)
- `Calculating Bonuses` § Flat Value … Exceptions: Ballistic Battery is "applied after base damage bonuses but before other bonuses/multipliers"; Volt's passive "after base damage and elemental bonuses but before critical damage multiplier and Sniper combo counter"; Energy Channel "after the critical damage multiplier". (Order of these three is wiki-stated; exact formulas are out of scope here.)

## 4. Physical damage mods (IPS)

**Formula:** `D_t = B_t × (1 + Σ base_dmg) × (1 + Σ physical_t)` for t ∈ {Impact, Puncture, Slash}.

- Physical mods scale only their own type: wiki `Damage` § Physical Damage (Piercing Hit +90 % on Reconifex 40 base, 60 % Puncture → +21.6; AX-52 100 % Puncture → +36); `Calculating Bonuses` § Calculating Physical Damage (Karak 7.3 Slash × (1 + 1.2)).
- Physical and elemental bonuses are "combined in the same step and are based only on the weapon's base damage and any mods that effect base damage" (`Calculating Bonuses` § Combining Physical and Elemental Damage).
- Arsenal display formula (`Damage/Calculation` § Total Damage):
  `Arsenal Total = B × [1 + Σ elemental + Imp% × ImpBonus + Punc% × PuncBonus + Slash% × SlashBonus] × (1 + Σ base_dmg) × [MS_base × (1 + Σ MS)]` (no faction; for melee drop multishot).

## 5. Elemental mods and element combination

### 5.1 Amount
**Formula:** `E_e = B_mod × Σ elemental_e%` where the sum is over every mod/buff of element e (or of the two primaries that combine into e).
- "Elemental mods apply to all damage done by a weapon" → percentage of total base (`Calculating Bonuses` § Calculating Elemental Damage; Karak 29 × 0.9 Heat). They use the Serration-modded base (`Calculating Bonuses` step 2; `Serration` § Notes). Elemental mods are "always applied at 100% value, even if the element is completely new" (`Damage` § Physical Damage).
- A combined element's value is the sum of its components: `Damage/Calculation` § Adding Physical and/or Elemental Mods item 5 (90 % Cold + 90 % Toxin → 180 % Viral).
- Innate element value of the same type simply adds (innate element is part of B).
- Progenitor bonus counts toward the base that elemental mods multiply (see § 3.2; wiki wording "elemental mods … will be affected"). Exact confirmation that the % uses `B_listed × (1 + valence)` rather than `B_listed` is from that wording only → treat as likely but UNVERIFIED numerically.

### 5.2 Combination order (wiki `Damage` § Modding)
1. Mod slots are read in order: "closest to top left (first) to the bottom right (last)"; `Calculating Bonuses`: "left to right, top row then bottom row". Exilus slot position in this order is not stated → UNVERIFIED.
2. Each new primary element pairs with the next uncombined primary in order; pairs: Heat+Cold = Blast, Electricity+Heat = Radiation, Heat+Toxin = Gas, Cold+Electricity = Magnetic, Cold+Toxin = Viral, Electricity+Toxin = Corrosive (`Damage` § Secondary Elemental Damage table).
3. A repeated element is placed at the position of its **first** occurrence (`Damage` § Modding: Thermite before Hellfire; `Calculating Bonuses`: "only the first is used when making combinations").
4. **Innate weapon elements go last**, after all mod slots (`Damage` § Modding; `Calculating Bonuses`). A mod of the same element as the innate element moves the innate element to that mod's position (Amprex + Stormbringer in slot 1).
5. Two innate primaries (weapon innate + progenitor bonus) are ordered by **HCET** (Heat > Cold > Electricity > Toxin): "Mod slot 1 → … → Mod slot 8 → Innate Heat → Innate Cold → Innate Electricity → Innate Toxin" (rendered Kuva/Tenet/Coda pages, transcluded from `Adversary System/Progenitor`; `Damage` § Modding says the same in words). The two innate elements "both behave as innate elements and combine with each other" if no mod element intervenes.
6. Innate **secondary** elements (Ogris/Penta Blast, Stug Corrosive, Nukor/Detron Radiation, Tenet Detron) are never broken up; primaries cannot combine into them; mod primaries combine among themselves independently (`Damage` § Modding). Tenet Detron + Electricity progenitor keeps Electricity separate from Radiation.
7. Riven with two element stats: priority goes to the **last** listed elemental stat on the riven (`Damage` § Modding, riven paragraph). If no other elemental mods, the riven's two elements combine with each other.
8. Combined primaries no longer deal their component damage or procs (`Damage` § Elemental Damage).

HCET table excerpt (rendered pages, columns = weapon innate element): Bonus Cold + modded Electricity → innate Heat: Radiation + Cold; innate Cold: Magnetic; innate Electricity: Magnetic; innate Toxin: Magnetic + Toxin. Bonus Toxin + modded Heat → innate Heat: Gas; innate Cold: Blast + Toxin; innate Electricity: Radiation + Toxin; innate Toxin: Gas. (Full 12×4 table in the cached HTML.)

Worked examples:
- Prova/Lecta (innate Electricity), mods Cold slot1, Toxin slot2, Heat slot3 → Viral (Cold+Toxin) + Radiation (Heat + innate Electricity) (`Damage` § Modding).
- Amprex (22 Electricity): Hellfire only → Radiation; Hellfire + Cryo Rounds → Blast + Electricity (`Calculating Bonuses`).
- Tenet Detron (26 Rad innate, +15.6 Elec progenitor, B = 41.6), Hellfire 90 % → Heat 37.44 combines with innate Electricity (last uncombined mod element + innate) → Radiation 15.6 + 37.44 = 53.04, which is the same type as innate Radiation 26 → total Radiation 79.04 per pellet. The merge of mod-made Radiation with innate Radiation into one type is implied by per-type damage, not stated → UNVERIFIED.

### 5.3 Elemental sources that are not mods
- `Calculating Bonuses` carries an UpdateMe: element-priority for non-mod elemental bonuses (abilities, shards) is undocumented. Violet Archon Shard: "+30 % (+45 %) Primary Electricity Damage … Electricity will combine with Mods", additive with Stormbringer (`Archon Shard` lines 390-392). Valence Formation / Nourish / Plunder / Crucible Overflow add combined or chosen elements directly (`Universal Weapon Bonuses` § Elemental Damage). Their position in the combine order: UNVERIFIED.

## 6. Faction damage (bane) bucket

**Formula:** hit damage × `(1 + Σ faction)`; DoT tick × `(1 + Σ faction)` **again**.
- Faction mods: ×1.05 per rank, 1.30 normal / 1.55 Primed; "total damage multiplier" (`Faction Damage Bonus` § intro/Mechanics). Multiplicative with Serration bucket (`Serration` § Notes: 2.65 × 1.3 = +244.5 %; Primed +310.75 %).
- Same bucket (additive with each other): Roar (Rhino 50 %×Strength; subsumed 30 %), True Master's Font Damage Blessing 25 %, rivens (base 45 %) (`Roar` info; `Universal Weapon Bonuses` § Faction Damage Bonus; `Faction Damage Bonus` § Other Sources). Roar example: `100 × 2.65 × (1 + 0.5 + 0.3)` with Bane of Grineer on Grineer.
- Faction mods do not apply to Corrupted or Narmer variants; Grineer/Corpus/Infested mods do apply to Kuva Grineer / Corpus Amalgam / Infested Deimos (`Faction Damage Bonus` § Mechanics).
- Faction is not shown in the arsenal (`Calculating Bonuses` step 3; `Damage/Calculation` § Total Damage).
- **Double dip on DoT: still documented as current.** `Damage over Time` § DoT Damage Scaling: "applied once more during the calculation of damaging status effects"; Bleed tick example `100 × 2.65 × 0.35 × 1.3² × 1.9`. `Damage/Slash Damage` § General: `Modded Base Damage = B × (1 + base) × (1 + faction)` then tick `= 0.35 × Modded Base × (1 + faction) × (1 + status dmg) × extra`; effective (1.3)² = +69 %, (1.55)² = +140.25 %. `Damage/Calculation` § Damage Over Time (page category Update 36) with Toxin example `(40 × 1.55 + 1) × 0.5 × 3.25 × 1.55 = 158.68`. Roar also double-dips (`Roar` info). U29.10 note "Fixed the Bane Mods applying the increased Damage twice" (`Faction Damage Bonus` § Patch Notes) refers to hit damage, not DoT; the DoT pages written later still show the square.

## 7. Multishot

**Formula:** `MS = MS_base × (1 + Σ MS_bonus)`; pellets per shot = `floor(MS)` plus one more with probability `frac(MS)`; expected pellets = MS (`Multishot` § Calculating Multishot; Lex +180 % → 2.8; Hek + Hell's Chamber → 15.4).
- Each pellet rolls crit and status independently (`Multishot` intro; `Critical Hit` intro).
- Arsenal damage shows the average (`Calculating Bonuses` § Multishot).
- Riven multishot curse cannot bring multishot below 1 (`Multishot` § Sources).
- Acuity mods (Primary/Pistol Acuity) **SET** multishot to base, ignoring all bonuses including negative ones (`Multishot` § Locking Multishot, U38; `Calculating Bonuses` Override row). As of U44 only Primary and Pistol Acuity exist.
- Continuous weapons: extra beams on the same target merge into one tick with damage **and** status chance summed, but crit chance of one beam; DoT output is thus affected twice by multishot; forced procs (Hunter Munitions etc.) apply after merging, once per tick (`Multishot` § Continuous Weapons).
- Spearguns: multishot does not affect the thrown component (`Multishot` § Spearguns).

## 8. Critical chance

**Formula:** `CC = CC_base × (1 + Σ relative) + Σ absolute` (`Critical Hit` § Relative / Absolute Increases; `Calculating Bonuses` § Flat Value).
- Relative (additive with Point Strike): mods, Arcane Hot Shot, Biting Frost, Blood Rush / Gladiator set (`relative += BloodRush × (Combo − 1)`), weak-point crit chance (Acuity, Worthy Comradery, Cyte-09 passive) on weak-point hits, Archon Shard secondary/topaz crit (`Critical Hit` § Blood Rush; `Primary Acuity` § Notes; `Archon Shard` lines 264-266, 368-370; `Universal Weapon Bonuses` § Critical Chance).
- Absolute ("percentage points", added after mods, operation type ADD): Arcane Avenger 45 %, Cat's Eye 60 % (does not work on abilities), Smeeta Charm 200 %, Harrow Covenant (≤ 50 % body / ×4 on weak points, cap 200 %), Kullervo Wrathful Advance (+50/100/150/200 % × Strength, melee only; subsumed 25-100 %), Shadow Haze, Secondary Enervate, Ballistic Bullseye, Concentrated Arrow, Piercing Navigator*, Crystallize, Shroud of Dynar, Hunter Synergy, Puncture "Weakened" (+5 %/stack, max 25 %, not AoE, not abilities) (`Critical Hit` § Sources of Absolute; `Arcane Avenger`; `Covenant`; `Wrathful Advance`; `Status Effect` line 40). *Piercing Navigator is the exception applied **before** other bonuses (`Calculating Bonuses` § Exceptions).
- Laetum "−10 critical chance percentage points" is an ADD too (`Calculating Bonuses` overview).
- Worked: Braton 12 % × (1 + 1.5) + 0.45 = 75 %; Atterax 25 % × 2.2 + 60 % + 45 % = 160 % (`Arcane Avenger` § Notes).

## 9. Critical damage multiplier and tiers

**Formula:**
```
CDM = Q(CDM_base + base_additions) × (1 + Σ relative) + Σ absolute
tier t: multiplier = 1 + t × (CDM − 1)
t = floor(CC) + [rand < frac(CC)]           (CC as a fraction, ≥ 0)
average multiplier = 1 + CC × (CDM − 1)       (exact for any CC, linear in t)
```
- Relative/absolute split: `Critical Hit` § Critical Damage. Absolute: Arcane Crepuscular (+0.5x … +3.0x at R5, "Final Critical Multiplier"), Cold "Freeze" (+0.1, +0.05/stack, +0.5 at 9 stacks, +1.0 frozen; not AoE), Shroud of Dynar, Tenacious Bond (+1.2x), Melee Doughty. Arcane Crepuscular page formula: `1 + tier × (CDM − 1 + bonus)` — consistent with adding absolute to CDM.
- Base CDM quantization: `Code(x) = floor(clamp(x,0,32) × 4095/32 + 0.5)`, `Q(x) = Code × 32/4095`, f32 arithmetic, applied "before mods, but after certain effects that are additive to the base value" (e.g. Incarnon "Increase Base Critical Damage Multiplier by +0.4x", which the Braton page says "is added as base critical damage and will be affected by mods") (`Critical Hit` § Quantization).
- Tiers: yellow (1), orange (2, CC > 100 %), red (3, > 200 %), red ! / !! / !!! (4-6+), tiers keep increasing beyond; display-only colour change in U21; tier system U13.5 (`Critical Hit` § Critical Tiers and Patch History).
- Tier modifiers: Vigilante set +5 % per mod (max 30 %) chance to raise tier by 1 on primaries; "Critical Promotion" weak points (Rogue Culverin canisters, Scaldra backpacks) raise tier by 1 (`Critical Hit` § Critical Tiers; `Enemy Body Parts` § Critical Promotion).
- Bug: tier multiplier past ~255 overflows the displayed number (display only) (`Critical Hit` § Bugs).

Worked example: base CC 30 %, Point Strike +150 %, Arcane Avenger +45 % → CC = 0.3 × 2.5 + 0.45 = 1.20. Base CDM 2.0 → Code = floor(2.0 × 127.96875 + 0.5) = 256 → Q = 8192/4095 = 2.000488. Vital Sense +120 % → CDM = 4.401074. Tier 1 (80 %) ×4.4011, tier 2 (20 %) ×7.8021; average 1 + 1.2 × 3.401074 = 5.08129 (= 0.8 × 4.4011 + 0.2 × 7.8021).

## 10. Headshots, weak points and head-crits

Three separate flags (`Enemy Body Parts` § Body Part Types): weak point, headshot, critical-headshot ("headcrit").

**Formula (one pellet hitting a body part with base multiplier P):**
```
part = P + k × Σ WP_damage_bonus          k = 1.5 if the part is also a headshot, 1.0 if not
head = (1 + Σ HS_multiplier_bonus)        only on headshot parts
crit = 1 + t × (h × CDM − 1)              h = 2 on headcrit parts, 1 otherwise
hit  = … × part × head × crit × (independent multipliers: Sonar, Sniper combo, Cernos Prime HS bonus)
```
- P: usually 3.0 on Grineer, Corpus, Infested, Sentient, Anarchs, Narmer, Corrupted heads; ~1.5 on Murmur and Techrot (`Enemy Body Parts` § Weak Point Hits). Base HS multiplier raised from 2× to 3× in U32 (`Enemy Body Parts` § Patch History). Older pages still say 2× (`Calculating Bonuses` § Multiplicative Stacking lists "Headshots (2x damage on headshot)") → outdated.
- WP damage bonuses (Primary/Pistol Acuity +350 %, Seek +75 %, Cascadia Accuracy, Worthy Comradery, riven splice) add to P at 1.5× rate on heads: `3 + 1.5 × (3.5 + 0.75) = 9.375`; non-head WP `1 + 3.5 + 0.75 = 5.25` (`Enemy Body Parts`; `Primary Acuity` § Notes).
- HS multiplier bonuses (Primary/Secondary Deadhead +30 % at R5, Prowl, Target Acquired, sniper zoom, Kuva Chakkhurr innate) add together and multiply the part term: `(3 + 1.5 × 4.25) × (1 + 0.3 + 0.4) = 15.9375`; Daikyu `3 × (1 + 0.3 + 0.75)` (`Enemy Body Parts` § Headshots and § Notes; `Primary Deadhead` § Notes). Cernos Prime's headshot bonus and Sonar/Sniper-combo multipliers are separate multiplicative factors.
- Headcrit: "Applies a 2x multiplier to the weapon's base critical multiplier"; `Critical Hit` § Critical Headshots: `HS × (1 + t × (2 × CDM − 1))`. Exceptions: no headcrit on Corpus humanoids/ground units, most Murmur, much of Techrot, MOA "fanny pack" (3× but no headcrit); Jordas Golem engines 1× but headcrit; Juggernaut maw/belly 4×, Ropalolyst head 0.2× crit factor (`Enemy Body Parts` § Critical Headshots; `Critical Hit`). "If a weapon defaults to a 1x headshot multiplier it will not benefit from the critical bonus, even if the headshot multiplier is raised" (`Critical Hit`).
- Weapons with 1× headshots ignore WP damage bonuses but still get Deadhead: Arca Plasmor (+Tenet), Alternox primary, Catchmoon, Ignis/Ignis Wraith, Lex/Paris/Dread Incarnon forms, Steflos, Electricity and Gas procs, etc. (`Enemy Body Parts` § Notes list). Radial/AoE damage: 1× and cannot trigger headshot conditions (U32).
- Status DoT inherits the body-part multiplier of the inducing hit (`Damage over Time` § DoT Damage Scaling) but **not** Sonar / Detect Vulnerability weak spots (`Damage/Slash Damage`).

Worked example (3× head that can headcrit, CDM 4.401074, Primary Acuity): part = 3 + 1.5 × 3.5 = 8.25; tier-1 crit factor = 2 × 4.401074 = 8.802149 → 72.618× base hit. Average crit factor on that head at CC = 1.20 (+Acuity WP crit chance ignored here): 1 + 1.2 × (8.802149 − 1) = 10.3626.

## 11. Condition Overload-type bonuses (interaction with buckets only)
- CO-type sources (+X % "Direct Damage" per status): Condition Overload 80 %, Galvanized Aptitude/Savvy 40 % ×2 stacks, Galvanized Shot 40 % ×3, Secondary Shiver 45 %/Freeze stack, Cedo 60 %, several Incarnon perks, Shattering Frost decree; they add with each other (`Condition Overload (Mechanic)` § Sources).
- Two behaviours, per attack (community catalogue on that page):
  - "Multiplying" (most projectile attacks, e.g. Arca Plasmor, Latron Incarnon): `B × (1 + Σbase) × (1 + ΣCO) × other`.
  - "Adding" (most hitscan, the common case): `B × [(1 + Σbase) × other_multipliers + ΣCO]` — the recalculation **omits** final multipliers (Extinguished Dragon Key, range falloff, Longbow Sharpshot, Primary Compression on projectiles, Duality, Furious Javelin, Garuda passive, Eclipse, Hall of Mirrors, Parasitic Link, Overdriver), Incarnon flat base damage, some stance multipliers and bow charge scaling.
  - Some attacks apply CO from the parent projectile or uncharged damage (Kuva Bramma cluster, Kulstar; Paris charged shot uses uncharged base).
- Not applied to radial explosions (with listed exceptions). Detailed per-weapon behaviour belongs to the CO research file; listed here because it decides which bucket CO lands in.

## 12. Other independent multipliers (each its own factor)
`Calculating Bonuses` § Multiplicative Stacking & `Universal Weapon Bonuses` § Damage Multipliers: Primed Chamber (2× first shot), Lasting Purity (1.6× aiming), Eclipse (Strength × 200 %, subsumed 30 %), Garuda passive (≤ 100 %), Nidus Parasitic Link (25 %), Vauban passive (25 %), Overdriver (25 %), Ivara Navigator (5×), Sonar, Sniper combo counter, stealth damage bonus, damage falloff, Extinguished Dragon Key (−75 %). Melee: combo/stance multipliers (`Damage/Calculation` § Melee DPS). "All sources are multiplicative to each other and to base damage bonuses, unless stated otherwise." Whether two items of this list ever share a bucket is not stated → assume independent (UNVERIFIED per item).

## 13. Status chance

**Formula:** `SC = SC_base × (1 + Σ relative) + Σ absolute` (per pellet).
- Relative: status mods (Rifle Aptitude, Galvanized Aptitude, dual-stat elemental mods), Crimson Archon Shard primary SC (additive with Rifle Aptitude) (`Status Effect` § Status Mods; `Archon Shard` lines 259-261).
- Absolute (ADD): Entropy Burst (`Calculating Bonuses` overview). Tau "Status Vulnerability" raises status chance **received** (+10 %/stack, ≤ 100 %), described as a "multiplier" — exact form UNVERIFIED (`Damage` § Unique Damage; `Status Effect` § Status Vulnerability).
- Per pellet: since U27.2 the arsenal SC is the per-pellet chance for shotguns too (`Status Effect` § Multishot, Trivia lines 879-883; U27.2 notes). Before U27.2: `per pellet = 1 − (1 − SC)^(1/pellets)`.
- Above 100 %: number of procs per pellet = floor(SC) + Bernoulli(frac(SC)); each proc's type drawn independently, so duplicates are possible (`Status Effect` § Status Chance; U27.2 notes). Expected procs per shot = `MS × (forced procs + SC)` (`Status Effect` § Average Procs; e.g. 1.9 MS × 150 % = 2.85).
- Proc type weight = type damage / total damage, using the damage composition (physical no longer ×4 since U27.2); enemy immunities remove types from the weights; resistances do not (`Status Effect` § Damage Distribution and § Status Immunity Interactions).
- Continuous weapons: merged beams sum SC (see § 7).

## 14. Damage quantization

**Rule (current):** per damage type, round to the nearest multiple of 1/32 of the attack's modded base damage, then apply all further multipliers.
```
Scale = B_mod / 32
x     = D_t / B_mod
Quantized(x) = sign(x) × floor(|x| × 32 + 0.5) / 32
D_t(q) = Quantized(x) × B_mod
```
- Source: `Damage/Calculation` § Quantization (exact rule with negative-value handling; 30/30/40 example; Nagantaka Prime example). Physical and elemental are quantized; each final element (sum of all its mod and bonus contributions) is quantized once as a whole ("Elements formed by a sum of mods and bonuses quantize their sum alone").
- **Grid changed from 1/16 to 1/32 in U40** ("(Undocumented) Damage Quantization changed from 1/16 to 1/32", `Damage` § Patch History). The original research (Theroxenes, forums 2018-03-02 / Reddit 2018-02-28) was 1/16. The task brief's "/16" is outdated.
- The page carries an UpdateMe "conflicting info": (a) base-damage and faction multipliers "only multiply final quantized values"; (b) they multiply both numerator and scale, hence are "a simple multiplier to any quantized total". Both lead to the same numbers for multipliers that scale every type equally, so the conflict only matters for wording; treat base-damage mods, faction, crit, headshot etc. as applied after quantization.
- Display: pop-ups show `floor(raw + 0.5)`; the raw value is not rounded (`Damage/Calculation` § Damage Over Time).
- DoT ticks are **not** quantized (they use modded base damage), use binary32 arithmetic, and the accumulator starts at 1: `tick = (Σ seeds + 1) × C × M`, C = 0.5 (Heat/Elec/Toxin/Gas) or 0.35 (Slash) (`Damage/Calculation` § Damage Over Time).
- Hitscan weapons do not quantize against Object health (Nullifier bubbles, Tornado); projectiles do (`Damage/Calculation` § Trivia). Galatine's 2.5 % Impact quantizes to 0 (cannot trigger Shattering Impact).

Worked example (30 I / 30 P / 40 S, base 100, Serration 165 %, Hellfire 90 %, Bane 30 %): scale 3.125 → I 31.25, P 31.25, S 40.625 (13 × 3.125), Heat 28.8 → 29 × 3.125 = 90.625; sum 193.75 per 100 base. × 2.65 = 513.4375; × 1.3 (faction) = 667.46875 (unquantized would be 654.55, +2.0 %). Nagantaka Prime check reproduced: Puncture 15.57 × 2.2 → 32.4375; Slash 155.7 → 156.78125; Viral 1.5 × 173 → 259.5.

## 15. Consolidated per-pellet formula (before enemy defenses)

For one pellet of one attack, damage type t:

```
B'_t      = B_t + X_t                                   # B_t listed base incl. progenitor bonus (§3.2); X_t Incarnon flat (§3.1, split UNVERIFIED)
B'        = Σ_t B'_t
D_t       = B'_t × (1 + P_t)            for t ∈ IPS      # P_t: physical mods of that type (§4)
D_t       = B'_t + B' × E_t             for elements      # E_t: Σ elemental % after combination (§5), innate element B'_t joins its combined bucket
D^q_t     = Quantize_1/32(D_t ; B')                       # §14
Hit       = Σ_t D^q_t
          × (1 + ΣBase)                                   # bucket A: Serration, Heavy Caliber, Vex Armor, Deadhead stacks, Arcane Fury… (§2)
          × CO_term                                       # bucket B: (1 + ΣCO) if "multiplying"; if "adding" replace A×B by [(A)×(final mults) + ΣCO]/(final mults) (§11)
          × (1 + ΣFaction)                                # bucket C: bane mods + Roar + Damage Blessing + riven faction (§6)
          × (P_part + k × ΣWP) × (1 + ΣHS)                # bucket D, E: body part (§10)
          × (1 + t × (h × CDM − 1))                       # bucket F: crit tier, h = 2 on headcrit (§9, §10)
          × Π independent multipliers                     # bucket G: Eclipse, Primed Chamber, Lasting Purity, Garuda, Sonar, sniper combo, stealth, falloff, stance/combo (§12)
```
with
```
CC  = CC_base × (1 + ΣCC_rel [+ ΣWP_CC on WP hits] [+ BloodRush × (combo − 1)]) + ΣCC_abs
CDM = Q(CDM_base + ΣCDM_baseAdd) × (1 + ΣCDM_rel) + ΣCDM_abs
SC  = SC_base × (1 + ΣSC_rel) + ΣSC_abs                 (per pellet)
MS  = MS_base × (1 + ΣMS)   (Acuity: MS = MS_base)
```
Expected damage per shot (all pellets hit, body shot, no CO): `MS × Hit_noncrit × (1 + CC × (CDM − 1))` (`Damage/Calculation` § Average Shot; `Critical Hit` § Average Damage).

Bucket citations: A `Calculating Bonuses` § Additive Stacking; B `Condition Overload (Mechanic)` § Math; C `Faction Damage Bonus`, `Roar`, `Universal Weapon Bonuses`; D/E `Enemy Body Parts`; F `Critical Hit`; G `Calculating Bonuses` § Multiplicative Stacking, `Universal Weapon Bonuses` § Damage Multipliers. Quantization position: `Damage/Calculation` § Quantization Summary ("Get Scale → Quantize Physical and Elemental → Add → Apply all other multipliers").

---

## Edge cases

1. Quantization grid is 1/32 since U40 (undocumented), not 1/16; tiny types (Nagantaka Impact 1 %) round to 0 and vanish (also from status weights? UNVERIFIED).
2. Mixed-type weapons gain or lose a few % from quantization; it matters for exact golden tests.
3. Base CDM is quantized to a 32/4095 grid (2.0 → 2.000488); Incarnon "+base CDM" is added before quantization and before mods.
4. Progenitor bonus: % of listed base, counts as base damage (Serration-scaled, feeds elemental %), placed after all mods in combine order, HCET order when weapon has its own innate primary, never merges into an innate secondary (Tenet Detron).
5. Progenitor bonus uses linear 100 %→0 % falloff over the weapon's falloff range regardless of the weapon's own falloff curve.
6. Valence fusion rounds down to 0.1 % and snaps ≥ 58 % to 60 %.
7. Same-element mod pulls an innate element forward in the combine order; repeated elements use the first slot.
8. Riven with two elements: the last-listed element has priority.
9. Unairu Impact buff behaves like an elemental (% of total base) even though Impact is physical.
10. Faction bonuses (bane, Roar, Damage Blessing) apply twice to status DoT; Roar is additive with bane, not multiplicative.
11. Faction mods ignore Corrupted/Narmer variants; do apply to Kuva Grineer / Amalgam / Deimos.
12. CO bonuses are additive with Serration on most hitscan attacks but multiplicative on many projectiles; additive CO ignores final multipliers and Incarnon flat base damage (Braton Incarnon Evo II).
13. Absolute crit chance (Avenger, Covenant, Wrathful Advance, Cat's Eye, Weakened) is added after relative mods; Piercing Navigator is added before.
14. Covenant crit bonus ×4 on weak points, capped 50 % body / 200 % WP; Cat's Eye does not work on abilities.
15. Weak-point damage bonuses count 1.5× on head weak points, 1.0× on non-head weak points (MOA rears, Bursa panels, Severed Warden arms since U44).
16. Head-crit doubles CDM only on narrow "headcrit" parts; Corpus humanoids, most Murmur/Techrot never headcrit; 1×-headshot weapons never headcrit even with Deadhead.
17. AoE/radial damage: 1× headshot, no headshot triggers, no CO, no Weakened/Freeze crit bonuses.
18. Electricity and Gas procs have 1× headshot per Body Parts notes, but Primary Deadhead page says they "benefit twice" from HS bonuses — contradictory.
19. Continuous weapons: multishot merges into one instance (damage and SC summed, CC not), DoT affected twice by multishot, forced procs once per tick.
20. Acuity mods lock multishot to base, ignoring riven penalties; riven MS curse cannot push MS below 1.
21. Status chance above 100 % can give duplicate proc types; immunities reshape proc weights, resistances do not.
22. Shotguns: SC is per pellet since U27.2 (old formula `1 − (1 − SC)^(1/n)` obsolete).
23. Crit tier can be raised by Vigilante set (5 %/mod) and "critical promotion" weak points; tier multiplier past ~255 breaks the displayed number.
24. Combined elemental DoTs (Gas, Blast) are only buffed by literal combined-element damage (Valence Formation, Leaded Gas), not by component mods; the reverse also holds (Corrosive mods buff a forced Toxin DoT from Toxic Lash).
25. Old pages still say 2× headshots (pre-U32) — use 3×.

## Open questions

1. How Incarnon flat "+X base damage" is split among damage types (proportional?) and whether it enters the quantization scale — UNVERIFIED.
2. Whether elemental % uses `B_listed × (1 + valence)` (progenitor included) — wiki wording says yes; no numeric test found — UNVERIFIED.
3. Position of non-mod elemental sources (Violet shard Electricity, Nourish, Plunder, Valence Formation, Crucible Overflow) in the combine order — UNVERIFIED (wiki UpdateMe).
4. Exilus slot and arcane-granted elements in the slot order — UNVERIFIED.
5. Headcrit "2× base critical multiplier": does it double absolute CDM bonuses (Crepuscular, Freeze) too, or only the modded CDM? `Critical Hit` uses total CDM; not tested — UNVERIFIED.
6. `Damage/Calculation` § Weak Point Hit uses `AvgShot × (WP mult + WP bonus)` and § DoT uses `WP mult × (1 + WP bonus)`; both contradict `Enemy Body Parts` (`P + 1.5 × bonus` on heads). Prefer Enemy Body Parts (U44-era) — flag for testing.
7. Electricity/Gas proc headshot behaviour (1× vs. gets Deadhead twice) — contradictory pages.
8. Quantization "conflicting info" UpdateMe: confirm faction/base multipliers are outside quantization for weapons with progenitor or Incarnon flat damage.
9. Tau Status Vulnerability: exact form (multiplier on SC vs. additive) — UNVERIFIED.
10. Whether every universal "weapon damage" buff (Holster Amp, Vigorous Swap, Provoked, Amp, Shooting Gallery) is in the Serration bucket — only Vex Armor and Deadhead are explicitly confirmed.
11. Release dates for U40 (quantization change), U42 (valence rolls), U44 (weak-point standardization) not on the pages read.
12. Incarnon-form status chance on shotguns (Strun/Boar) scaled by base pellet count — seen only in the inspiration changelog; wiki source not yet found.

## Sources used
- wiki.warframe.com pages (raw wikitext, cached under `research/external/cache/wiki/`): Damage, Damage/Calculation, Damage/Slash Damage, Damage/Heat Damage, Damage over Time, Critical Hit, Status Effect, Multishot, Enemy Body Parts, Calculating Bonuses, Faction Damage Bonus, Serration, Galvanized Shot, Condition Overload (Mechanic), Kuva Weapons, Tenet Weapons, Coda Weapons, Valence Fusion, Incarnon, Braton Incarnon Genesis, Lex Incarnon Genesis, Arcane Avenger, Arcane Crepuscular, Roar, Covenant, Wrathful Advance, Primary Acuity, Primary Deadhead, Archon Shard, Universal Weapon Bonuses.
- Rendered (api.php parse) pages: Kuva Drakgoon, Coda Hema, Tenet Detron (`*.parsed.html` in the cache) for progenitor notes and the HCET table.
- Forum (anecdotal): [forums.warframe.com topic 1357504](https://forums.warframe.com/topic/1357504-how-does-the-new-perk-fatal-affliction-of-soma-work) on Incarnon flat base damage applying before mods.
