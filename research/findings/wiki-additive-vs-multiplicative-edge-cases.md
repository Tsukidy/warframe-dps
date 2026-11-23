Status: draft

# Additive vs. multiplicative damage bonuses, Condition Overload-type bonuses and their exceptions (game-truth side)

Scope: which damage bonus lands in which bucket of the per-hit formula, with emphasis on bonuses whose bucket changes per weapon or per attack (Condition Overload and its gun counterparts first of all). The core formula, quantization, crit tiers, headshot/weak-point math and status-chance math are in `research/findings/wiki-core-damage-formula.md`; this file only repeats them where a bucket decision depends on them.

Research date: 2026-11-23 (game state late 2026). All wiki pages were read as raw wikitext (`?action=raw`) and are cached in `research/external/cache/wiki/` (file name = page name, `/` → `__`). "U" = game Update number as tagged on the wiki by `{{ver|…}}`. The wiki gives update numbers, not dates, for most entries.

Freshness check (MediaWiki API, `prop=revisions`): `Condition Overload (Mechanic)` last edited 2026-11-23 (actively maintained; its Math section says "Mechanics below confirmed as of 2025-11-03"); `Eclipse` 2026-10-25; `Secondary Shiver` 2026-08-31; `Galvanized Aptitude` 2026-08-10; `Calculating Bonuses` 2026-06-27; `Galvanized Shot` 2025-01-06.

Notation used below:
- `B` = listed base damage of the attack (per pellet), including any Kuva/Tenet/Coda progenitor bonus.
- `X` = Incarnon "Increase Base Damage by +X" flat addition.
- `A` = Σ of additive base-damage bonuses (the "Serration bucket").
- `C` = Σ of CO-type bonuses currently active (per-status bonus × number of unique statuses, plus per-stack/per-condition bonuses).
- `B_co` = the damage value the game multiplies by `C` ("CO base"); equals `B` only for well-behaved attacks.
- `F` = product of "final damage multipliers" that the additive-CO recalculation omits (list in §1.6).
- `R` = everything applied after the hit's base value: elemental/physical composition, faction, body part, crit, etc.

---

## 0. Bucket model (summary)

| Bucket | Stacking inside bucket | Stacking with other buckets | Source |
|---|---|---|---|
| Base damage ("Serration bucket", "+X% Damage", "Melee Damage", "Weapon Damage" on guns) | additive (`STACKING_MULTIPLY`) | multiplies everything else | `Calculating Bonuses` § Overview, § Additive Stacking |
| CO-type ("+X% Direct Damage" per status/condition) | additive with each other | **per attack**: additive with base bucket (most hitscan, most melee) **or** a separate multiplier (most projectiles); see §1 | `Condition Overload (Mechanic)` § Sources, § Math, § Attack Catalog |
| Physical (Impact/Puncture/Slash %) and elemental % | additive per type; applied to the base-modded value | multiplies with base bucket (elemental % is of base-modded damage) | `Calculating Bonuses` § Damage Application Order |
| Faction ("Damage to X", ×1.30 / ×1.55, Roar, Damage Blessing, riven faction) | additive | separate multiplier; applied again to status DoT | `Faction Damage Bonus` § Mechanics; `Roar`; `Universal Weapon Bonuses` § Faction Damage Bonus |
| Independent multipliers (Eclipse, Garuda passive, Parasitic Link, Vauban passive, Overdriver, Navigator, Primed Chamber, Lasting Purity, Longbow Sharpshot, Primary Compression on projectiles, sniper shot combo, heavy-attack combo multiplier, heavy-attack type multiplier, Slam Damage, Power Throw on glaives, Furious Javelin, Duality, Hall of Mirrors, Extinguished Dragon Key, falloff) | each its own factor ("All sources are multiplicative to each other and to base damage bonuses, unless stated otherwise") | multiply | `Universal Weapon Bonuses` § Damage Multipliers; `Calculating Bonuses` § Multiplicative Stacking; individual pages cited below |
| Crit chance relative (Point Strike, True Steel, Blood Rush, Gladiator set, Archon shard, Arcane Hot Shot, Argon Scope/Galvanized Scope, Secondary Outburst, Secondary Kinship, Melee Animosity, weak-point crit) | additive | `CC = CC_base × (1 + Σrel) + Σabs` | `Critical Hit`; pages cited in §2, §5, §6 |
| Crit chance absolute (Arcane Avenger, Cat's Eye, Covenant, Smeeta Charm, …) | additive | added after relative | `Calculating Bonuses` § Flat Value; `Arcane Avenger` § Notes |
| Crit damage relative (Vital Sense, Organ Shatter, Crimson/Violet shard, Secondary Outburst, Primary Frostbite, Galvanized Steel) | additive | multiplies base CDM | `Archon Shard` lines 254-256, 394-397; arcane pages |
| Crit damage absolute (Arcane Crepuscular "Final Critical Multiplier", Tenacious Bond) | additive | added after relative | `Arcane Crepuscular` § Notes; `Universal Weapon Bonuses` § Absolute Critical Damage |
| Status damage (Elementalist mods, Emerald shard Toxin, Galvanized Elementalist) | additive | multiplies DoT ticks only | `Archon Shard` line 337; `Damage over Time` § DoT Damage Scaling |
| Extra Hit (Toxic Lash, Xata's Whisper, Resupply, Silken Stride, …) | separate damage instance computed from the finished hit | not part of the hit's buckets | `Extra Hit` § Damage Calculation |

Order of the base stat (from `Calculating Bonuses` § Order of Operations): `[Base × Π(1 + Σ additive bonuses in each bucket)] + Σ flat bonuses`.

---

## 1. Condition Overload-type ("CO", "GunCO") bonuses

### 1.1 Sources and values (max rank)

All of these are "internally considered the same effect" and add with each other (`Condition Overload (Mechanic)` § Sources of Condition Overload-Style Bonuses):

| Source | Bonus | Stacks / condition | Notes / source |
|---|---|---|---|
| Condition Overload (melee mod) | +80 % per unique status | — | `Condition Overload` stats; 120 % → 80 % in U30.5; exponential `1.6^n` before U26 |
| Galvanized Aptitude (rifle) | +40 % per status per stack | 2 stacks on kill, 20 s → +80 %/status | `Galvanized Aptitude` stats; +80 % status chance |
| Galvanized Savvy (shotgun) | +40 % per status per stack | 2 stacks, 20 s → +80 %/status | `Galvanized Savvy` stats |
| Galvanized Shot (pistol) | +40 % per status per stack | **3** stacks, 14 s → +120 %/status | `Galvanized Shot` stats |
| Secondary Shiver (arcane, pistol or archgun) | +45 % per **Cold stack** on target | up to 10 Cold stacks → +450 % | `Secondary Shiver` stats; counts stacks, not status types |
| Shattering Frost (decree) | +80/160/240 % vs Cold-affected | — | `Decree` § Shattering Frost ("same behavior as Secondary Shiver"); also affects non-exalted abilities, additive with Archon-shard ability damage (`Condition Overload (Mechanic)` line 39) |
| Cedo / Cedo Prime innate | +60 % per status | — | `Condition Overload (Mechanic)`; `Cedo` § Notes |
| Dual Toxocyst Incarnon (Evo 2 Carnage Reign) | +33 % per status | — | `Condition Overload (Mechanic)` line 30 |
| Lato Incarnon (Evo 4 Carnage Reign) | +40 % per status | — | idem |
| Latron Incarnon (Evo 2 Swift Punishment) | +30 % per status | — | idem |
| Angstrum / Zylok (Evo 4) / Despair (Evo 2) / Soma (Evo 4) Incarnon "Fatal Affliction" | +40 % per status | — | idem |
| Kunai Incarnon (Evo 2 Swift Conclusion) | +100 % vs < 50 % health | — | idem line 37; **the attack table for Kunai says "+200 % (3x) bonus vs < 50 % hp"** → value conflict, UNVERIFIED |
| Dread Incarnon (Evo 2 Hitman's Opportunity) | +100 % vs < 50 % health | — | idem line 38 |

Not in the category: Lavos' Catalyze (exponential per condition and radial) (`Condition Overload (Mechanic)` line 43). Galvanized Diffusion/Chamber/Hell (multishot), Galvanized Scope/Crosshairs (crit chance while aiming), Galvanized Acceleration (projectile speed), Galvanized Steel (crit chance + crit damage on kill), Galvanized Elementalist (status damage + status chance), Galvanized Reflex (initial combo) are **not** CO-type; they are ordinary bonuses in their stat's bucket (pages `Galvanized_*`, see master table).

### 1.2 Counting and timing

- Counted statuses: every status with a damage-type equivalent (Impact, Puncture, Slash, Cold, Electricity, Heat, Toxin, Blast, Corrosive, Gas, Magnetic, Radiation, Viral, Void, Tau) plus hidden ones: Lifted or Knockdown (mutually exclusive) and Microwave (Nukor/Kuva Nukor, infinite duration) (`Condition Overload (Mechanic)` § What Counts As A Status Effect; `Condition Overload` § Notes).
- Count is of **unique types**, not stacks (except Secondary Shiver/Shattering Frost, which use Cold stacks/Cold presence).
- Maximum: Galvanized pages say 16 (they do not list Tau); the CO mod page and Mechanic page include Tau. 13 elemental/physical + Void + Tau + Lifted/Knockdown + Microwave = 17 → the "16" figure is from a list without Tau. UNVERIFIED which is right; practically irrelevant.
- Status source does not matter (other weapons, abilities, companions, allies, environment) (`Galvanized Shot` § Notes).
- Stacks of the Galvanized buff come only from kills by the equipped weapon (status kills count); when the buff times out, one stack is lost and the timer resets (`Galvanized Shot`/`Aptitude`/`Savvy` § Notes).
- **Timing:** the bonus of a new status is not applied to the hit that applied it, only to later hits. Multishot pellets are treated as **sequential** hits, so later pellets of the same shot benefit from statuses applied by earlier pellets (`Galvanized Aptitude` § Notes example: 2 stacks, base 100, Serration, MS 3 → 265 + 345 + 425 = 1035 instead of 795). Gunblade pellets likewise hit consecutively and can be amplified by earlier pellets of the same shot (`Condition Overload` § Notes).
- Before U31.1.6, multishot multiplied the GunCO bonus per pellet; fixed (`Galvanized Shot` patch history U31.1.6; `Condition Overload (Mechanic)` patch history).

### 1.3 Which damage instances get CO

`Condition Overload (Mechanic)` § Mechanics ("Direct Damage" wording, U30.9 description change):
- Applies: hitscan; hitscan explosion **only on the directly hit target**; hitscan ricochet (Lato Incarnon); projectiles (incl. homing, bouncing, punch-through, wave projectiles such as Arca Plasmor); embedded cloud only on the embedded target (Pox); beams, beam AoE (Glaxion Vandal), beam chain (Kuva Nukor), multi-beam (Ocucor).
- Blast, Electricity and Gas procs are generated with the bonus of the initial target and carry that damage to their radius (also `Condition Overload` § Notes: radial statuses get CO "as it is part of Modded Base Damage", computed on the initial target).
- Does not apply: projectile explosion radius (Ogris), hitscan explosion radius on non-direct targets (Ambassador), embedded cloud radius on non-direct targets (Torid). Melee: slams, heavy slams, radial explosions (glaive heavy-attack explosion) (`Condition Overload` § Notes).
- "Should work but doesn't": Balefire Charger (no direct-hit damage), Stug (entirely radial incl. Incarnon form), Ferrox embedded alt-fire, Azima turret, Sonicor, Scyotid alt-fire grab, Coda Bassocyst alt-fire, Proboscis Cernos radial and tendrils, Evensong charged radial, Tenet Tetra alt-fire AoE, Vadarya Prime lightning strikes, Zhuge Prime embedded explosion, Akarius Prime detonation, Castanas / Sancti Castanas detonations, Kompressa Prime radial (`Condition Overload (Mechanic)` § Attack Catalog rows with "Does not apply").
- Archguns: as of U43 only Secondary Shiver and Shattering Frost (Koumei) can give CO to archguns (`Condition Overload (Mechanic)` § Archguns).
- DE's stated reason for excluding AoE (U30.9 notes, quoted on all Galvanized pages): AoE weapons dominate usage; "we are not willing to further bolster AoE at this time".

### 1.4 The two math behaviours

Wiki: `Condition Overload (Mechanic)` § Application Behaviors and § Math (section marked "confirmed as of 2025-11-03").

**"Multiplying"** (behaviour #1; most projectile attacks):
```
Hit = B' × (1 + A) × (1 + C) × F × R          (B' = B + X when the attack's CO uses the Incarnon flat; see §1.7)
```
"A multiplicative bonus that additively stacks with only CO-like damage bonuses AND multiplicatively stacks with other damage bonuses (e.g. Serration, Vex Armor, Arcane Fury). Its functionality is reliable and compatible with all weapon bonuses." The wiki's example: `Base × (1 + 1.65) × (1 + 0.4 + 0.4)`.

**"Adding"** (behaviours #2/#3/#4; the common case, most hitscan and melee):
```
Hit = [ (B + X) × (1 + A) × F  +  B_co × C ] × R
```
Wiki form: `(Base × Most Damage Bonuses × Other Damage Multipliers) + (Base × CO)`, i.e. `Base × [(Most Damage Bonuses × Other Damage Multipliers) + CO]`. "An additive recalculation of all effects, but omits +X% Damage in the recalculation, thus making CO mathematically additive to +X% Damage. However, this recalculation inadvertently omits some other effects."
- #3 "intended": `B_co = B` (Braton normal and Incarnon, Latron normal, Paris uncharged, almost all hitscan).
- #2 "better than expected": `B_co > B` (child projectiles use the parent projectile's damage: Kuva Bramma cluster contact, Kulstar cluster bombs; hitscan-AoE attacks use the hitscan damage).
- #4 "worse than expected": `B_co < B` (bows and charge weapons use uncharged damage; Incarnon flat excluded).
- #5: CO not applied at all (§1.3).

What `R` contains for the CO part: the CO mod page gives the melee formula `Total = Base × [1 + DamageMods + CO × n] × (1 + ElementalMods)`, so elemental/physical % scale the CO part (`Condition Overload` § Notes). That crit, faction and body-part multipliers also scale the CO part is how every wiki example is framed (they are applied "after"), but no page states it explicitly → **UNVERIFIED** (very likely; flag for a test).

### 1.5 Why it happens (as far as sources say)

- Adding: the CO bonus is computed as a **separate recalculation** of the attack's damage from a stored "base damage" value and is added to the normal hit; that recalculation does not include `+X% Damage` mods (hence additive with them) and also misses a set of later multipliers (§1.6) (`Condition Overload (Mechanic)` § Math).
- Which value is used as base: DE, U31.1: "A previous change had them operate relative to 'base damage' but the code was incorrectly getting base damage from the impact behavior rather than the projectile. This problem was pervasive and there are hundreds of weapons in our game!" Before U31.1 many projectiles used a default CO base of 10 (`Condition Overload (Mechanic)` patch history; `Galvanized Shot` U31.1).
- Wiki speculation (`Condition Overload (Mechanic)/Trivia`, marked `{{Speculation}}`): projectile weapons store their arsenal damage in `fire:LotusWeaponProjectileFireBehavior.projectileType.AttackData`, which overrides `impact:LotusWeaponImpactBehavior.AttackData`; CO reads one of these objects, causing the discrepancies. Pre-U31.1 Astilla case: CO scaled the hidden 3 Corrosive impact damage instead of 70 Impact → `70 + 3 × 1.8 = 75.4` instead of `70 × 1.8 = 126`. Why projectile attacks end up multiplicative is **not** explained by any source → UNVERIFIED mechanism.
- DE has never stated the multiplicative cases are intended; the Mechanic page's writers call much of it bugs that have persisted for years (`Condition Overload (Mechanic)` § Writer(s) Notes).

### 1.6 Multipliers omitted by the "Adding" recalculation (`F`)

From `Condition Overload (Mechanic)` § Math (these multiply the non-CO part only):
- Extinguished Dragon Key (−75 %)
- Range-based damage falloff (**but beam ramp-up is included**, i.e. ramp-up scales the CO part too)
- Longbow Sharpshot (also `Longbow Sharpshot` § Notes: `Damage bonus% = (1 + Serration) × (1 + Longbow Sharpshot) + Galvanized Aptitude`)
- Primary Compression on projectile weapons
- Equinox Duality; Excalibur Furious Javelin (`Furious Javelin` § Notes: "does not multiply with Condition Overload's bonus when additive"); Garuda passive; Mirage Eclipse (`Eclipse` § info: "The one flat damage buff that is additive with Eclipse comes from condition overload effects on weapons that have additive CO. Base damage is multiplied separately and then added together."); Mirage Hall of Mirrors damage modifier; Nidus Parasitic Link; Vauban Overdriver
- Incarnon Genesis flat base-damage increases (`B + X` vs `B`)
- Some melee stance multipliers on multi-hit inputs (§1.9)
- "Some natural weapon stat modifiers, such as Bow charging and more."

For "Multiplying" attacks all of these multiply the whole hit, CO included.

### 1.7 CO base (`B_co`) rules

| Rule | Evidence |
|---|---|
| Bows (Cernos, Cernos Prime, Rakta/Mutalist Cernos, Daikyu Prime, Paris/Paris Prime/Mk1-Paris, Dread charged, Kuva Drakgoon charged) use **uncharged** damage = 50 % of full-charge damage (bows have an innate 2× full-charge multiplier) | Catalog rows "CO-bonus only applies to base (uncharged) damage" |
| Partially charged base for Lanka (200 of 525 = 38 %), Miter (200 of 500 = 40 %), Drakgoon (400 of 700 = 57 %), Evensong (325 of 500 = 65 %), Ballistica (25 %), Ballistica Prime (50 %), Rakta Ballistica (25 %) | Catalog |
| Corvas (atmosphere) charged hitscan **respects** the charge bonus (Adding, 100 %) | Catalog § Archguns |
| Child projectiles use the parent projectile's damage: Kuva Bramma cluster contact, Kulstar cluster bombs (75 → 200), Sporelacer (secondary) bomblet (9 → 57), Zarr child bomb (15 → 25), Kuva Zarr child bomb (15 → 50); Mutalist Cernos charged toxin cloud uses the 205 uncharged arrow damage (5 → 205) | Catalog |
| Hitscan-AoE attacks: CO on the radial hit only for the directly hit target and scaled from the hitscan damage, not multishot-scaled: Ambassador alt (800 → 600), Braton-family Incarnon radial (74 → 70), Burston Incarnon radial (55 → 13), Ferrox (100 → 350), Tenet Ferrox (60 → 200), Glaxion Vandal (29 → 29), Opticor (400 → 1000), Opticor Vandal (200 → 400), Trumna (50 → 82), Zylok Incarnon radial (776 → 700), Mausolon main (250 %) and alt (33 %) | Catalog |
| Incarnon "+X base damage" is **excluded** from `B_co` on Adding attacks: Atomos (100 vs 124), Bronco Prime (238 vs 448), Cestra (26/36, 50/60), Despair (60/120), Dual Toxocyst (75/135), Furis (100/128), Lato Vandal (152/174), Lex Prime (1200/1220), Vasto Prime (420/564), Zylok Prime (500/530); Braton Prime Incarnon Evo II likewise (`Braton Incarnon Genesis` § Known Bugs) | Catalog "CO-bonus does not use base damage increase Evolution" |
| On some **Multiplying** Incarnon attacks the flat **is** included: Kunai / Mk1-Kunai Incarnon ("CO-bonus DOES use base damage increase Evolution; does not factor +200 % (3x) bonus vs < 50 % hp targets, effectively additive with it") | Catalog |
| Kuva/Tenet progenitor bonus **is** included (fixed: Lich hitscan and melee U31.1.7; projectiles U31.2/U31.5; "never broken on multiplicative cases") | `Condition Overload (Mechanic)` and `Galvanized Aptitude` patch history |
| Odd fixed values: Acceltra 35 → 26 (74.3 %); Sporelacer (secondary) 90.5 → 57 "for any configuration"; Noctua primary 250 → 200, alt 2750 → 2500, Final Verse Wordwarden 125 → 200, none scaling with Ability Strength; Artemis Bow scales off 75 per arrow instead of 240 and ignores Strength (and does not work with Concentrated Arrow); Exalted Umbra Blade light waves 250 → 500 (Adding) while heavy waves are Multiplying; Argo & Vel thrown glaive 600 → 200 (also ignores melee combo); Arum Spinosa heavy daggers 20 %/10 %, Quassus Prime 40 %/20 % (also ignore combo); Destreza spectral rapiers 200 %; Edun heavy projectile 25 %; Stropha heavy 25 %; Innodem aerial projectile 200 %; Zenistar heavy disc 75 → 780 (1040 %); Tenet Agendus heavy waves 20 % but Multiplying; Pox DoT cloud 20 → 50 (spore-impact value, recalculates every tick); Tenet Detron "only 65 % effective on projectiles created by Multishot"; Zhuge Prime direct hit 48.4375 after quantization | Catalog |

### 1.8 Worked examples

1. Adding vs Multiplying, same numbers. `B = 100`, Serration `A = 1.65`, Galvanized Aptitude 2 stacks and 3 statuses → `C = 0.8 × 3 = 2.4`.
   - Adding: `100 × (1 + 1.65) + 100 × 2.4 = 265 + 240 = 505`.
   - Multiplying: `100 × 2.65 × 3.4 = 901`.
2. Adding with Eclipse (×3 at 100 % Strength): `100 × [2.65 × 3 + 2.4] = 100 × 10.35 = 1035`; if CO were inside Eclipse it would be `505 × 3 = 1515`. Multiplying attack: `901 × 3 = 2703`.
3. Incarnon flat (Furis Incarnon, Adding): `B = 100`, `X = 28`, Hornet Strike `A = 2.2`, Galvanized Shot 3 stacks, 3 statuses `C = 1.2 × 3 = 3.6`: `128 × 3.2 + 100 × 3.6 = 409.6 + 360 = 769.6`; naive `128 × (1 + 2.2 + 3.6) = 870.4` (−11.6 %).
4. Bow (Paris charged, Adding): listed 320, `B_co = 160`, Serration, Galvanized Aptitude 2 stacks, 2 statuses `C = 1.6`: `320 × 2.65 + 160 × 1.6 = 848 + 256 = 1104`; naive `320 × 4.25 = 1360`.
5. Child projectile (Kuva Zarr child bomb, Adding): `B = 15`, `B_co = 50`, Serration, `C = 1.6`: `15 × 2.65 + 50 × 1.6 = 39.75 + 80 = 119.75` (naive `15 × 4.25 = 63.75`).
6. Hitscan AoE (Opticor charged radial on the directly hit target, Adding): `B = 400`, `B_co = 1000`, Serration, `C = 1.6`: `400 × 2.65 + 1000 × 1.6 = 1060 + 1600 = 2660`; every other target in the radius: `400 × 2.65 = 1060`.
7. Projectile Multiplying (Arca Plasmor): `600 × 2.65 × (1 + 1.6) = 4134`.
8. Longbow Sharpshot (+300 %) on an Adding bow (e.g. Daikyu per `Longbow Sharpshot` § Notes): bonus factor `(1 + 1.65) × (1 + 3) + 1.6 = 12.2` instead of `(1 + 1.65 + 1.6) × 4 = 17`. (The bow-uncharged `B_co` rule would further reduce the CO part; the Longbow page's formula does not show it → combination UNVERIFIED.)
9. Melee CO (Adding, 100 % stance input): `B = 200`, Primed Pressure Point +165 %, CO 4 statuses `C = 3.2`, Fever Strike +90 % Toxin: `200 × (1 + 1.65 + 3.2) × (1 + 0.9) = 200 × 5.85 × 1.9 = 2223` (`Condition Overload` § Notes formula).

### 1.9 Per-attack catalogue (condensed)

Source for everything in this subsection: `Condition Overload (Mechanic)` § Attack Catalog (community-tested; spreadsheets by Prof_Blocks_007, Rainy, WFCD linked as refs). "Anything not listed should be assumed to be Additive with +100 % bonus". Ratios are `B_co / B`.

**Multiplying (CO is a separate multiplier, ratio 100 %):**
- Primary: Aeolak (main 60, alt 97); Alternox; Alternox Prime; Arca Plasmor; Tenet Arca Plasmor; Basmu main; Battacor main; Bubonico main and alt; Buzlok main and alt; Catchmoon (Primary, Tremor); Cedo/Cedo Prime alt-fire; Coda Bassocyst main; Coda Hema; Hema; Mutalist Cernos uncharged arrow and uncharged toxin cloud; Cinta uncharged/charged/perfect; Dread Incarnon; Exergis; Felarx normal and Incarnon; Fulmin semi; Harpak alt; Javlok main; Latron/Latron Prime Incarnon (AoE gets no CO at all); Miter Incarnon; Nataruk charged and perfect; Nunchasa alt; Paracyst alt; Paris/Paris Prime Incarnon; Phantasma / Phantasma Prime alt child bomb; Quellor alt; Scourge / Scourge Prime throw; Shedu; Sporelacer (Primary, Tremor); Stahlta main and alt; Steflos / Steflos Prime; Tenet Envoy; Tombfinger (Primary, Brash); Torid main and toxin cloud; Trumna alt; Zenith alt disk.
- Secondary: Akarius Prime rocket impact; Aegrit direct; Angstrum/Prisma Angstrum Incarnon; Ballistica Prime Incarnon; Catabolyst reload impacts; Catchmoon (Secondary); Cyanex impact and burst; Epitaph charged and uncharged direct; Kunai/Mk1-Kunai Incarnon; Tenet Plinx alt impact (scales with magazine size); Seer; Kuva Seer; Sepulcrum primary and lock-on; Tenet Diplos lock-on; Tenet Spirex slug; Tombfinger (Secondary); Zakti/Zakti Prime; Zymos homing spore.
- Melee: Corufell heavy projectile; Exodia Contagion aerial projectile (embed/AoE no CO); Nepheri neutral fireball; Syam heavy waves 1 and 2; Tenet Agendus heavy waves (at 20 % ratio); Tenet Grigori energy disk (does not scale with melee base-damage sources such as Pressure Point or Arcane Fury, but universal weapon damage bonuses — Arcane Arachne, Deathbringer — do apply and are multiplied by CO); Verdilac energy wave.
- Archgun: Arbucep direct (all 6 projectiles; AoE 228 no CO); Kuva Ayanga direct; Corvas Prime uncharged/charged; Grattler; Larkspur Prime alt; Mandonel uncharged; Velocitus uncharged/charged.
- Exalted: Exalted Umbra Blade heavy waves.
- Robotic: Burst Laser family; Cryotra direct (cloud no CO); Helstrum; Laser Rifle; Prime Laser Rifle; Multron; Vulklok.
- Amps: Shwaak Prism, Phahd Scaffold, Shraksun Scaffold.
- Also Multiplying per other pages: Cedo alt-fire direct hits (`Cedo` § Notes: "additive with Galvanized Savvy, and multiplicative to normal base damage bonuses like Point Blank"); Secondary Shiver is multiplicative on Cyanex, Catchmoon, Epitaph, Seer series, Sepulcrum, Tenet Spirex (`Secondary Shiver` § Notes) — consistent with the catalogue.

**Adding but with ratio ≠ 100 %:** see §1.7 table (bows, partial charge, child projectiles, hitscan AoE, Incarnon flat, Noctua, Artemis Bow, odd melee projectiles).

**Adding at 100 % but listed as discrepancies vs the projectile pattern:** Acrid, Mara Detron, Tenet Detron (burst, but 65 % on multishot projectiles), Kunai normal, Laetum normal and Incarnon ("CO-bonus multiplies properly with Devouring Attrition perk"), Tysis ("does not affect DoT"), Zhuge Prime direct, Fluctus, Mandonel charged (hitscan), Stinger (not embedded DoT), Vulcax (500 + 100 → 1000, 167 %).

**Not tested yet** (`Condition Overload (Mechanic)/Testing` checklist, blank entries): Primary — Coda Sporothrix, Coda Synapse, Haalvu, all Kuva primaries (Bramma, Chakkhurr, Drakgoon, Hek, Hind, Karak, Kohm, Ogris, Quartakk, Sobek, Tonkor, Zarr), Tenet Arca Plasmor, Tenet Envoy, Tenet Ferrox, Tenet Flux Rifle, Tenet Glaxion, Tenet Quanta, Vinquibus. Secondary — Coda Pox, Coda Tysis, Dual Coda Torxica, EFV-8 Mars, Grimoire, Kuva Brakk, Kuva Kraken, Kuva Nukor, Riot-848, Sagek Prime, Scyotid, Tenet Cycron, Tenet Plinx, Tenet Spirex, Vesper 77. Melee — Amanata, Azothane, Ceramic Dagger, all Coda melee, Dorrclave, Dual Ichor, Dual Viciss, Galariak Prime, Harmony, Kestrel Prime, Redeemer/Redeemer Prime, Ruvox, Sampotes, Sarofang/Prime, Sarpa, Spinnerex, Sun & Moon, Thalys, Tonkkatt, Vastilok, Vericres, Vitrica. (Some of these nevertheless appear with results in the catalogue, e.g. Tenet Arca Plasmor, Tenet Spirex, Kuva Zarr — the two pages are not in sync.)

### 1.10 Melee stances

`Condition Overload (Mechanic)` § Stances: per stance and combo input, the CO contribution relative to expectation. 100 % = stance multipliers work as expected with CO; < 100 % worse (the recalculation omits the stance multiplier on some hits of multi-hit inputs), > 100 % better. Examples: Gnashing Payara neutral 36 %, heavy 40 %; Fracturing Wind block 25 %; Mountain's Edge heavy 233 %; Galeforce Dawn heavy 167 %; Exalted Blade stance 122–130 % on every tested input; High Noon and Bullet Dance (gunblades) 100 % everywhere; Desert Wind 50–86 %. The full 48-row table is in the cached page; it should be imported as data.

Model (proposed): `Hit_input = B × [S × (1 + A) × F + r × S × C] × R`, where `S` = stance multiplier of the hit and `r` = table ratio. Whether `r` is averaged over all hits of the input or applies per hit is not stated → UNVERIFIED.

CO mod note (U31.1.3 fix): "Condition Overload was not accounting for Melee weapons that deal different amounts of damage with different swings via combos" (`Condition Overload` patch history). The stance table is newer than that fix (it references current stances), so the residual discrepancies are current.

### 1.11 Incarnon flat damage × CO: the "bug" the inspiration simulates

- Inspiration changelog 2025-03-12: "Added a hidden bug simulation where the Incarnon perk 'increase Damage by +x' when used with mods like 'GunCO' does not fully work with them on some attacks" (`research/external/inspiration/guide.txt` line 329, read only to identify the mechanic).
- Game-truth side: confirmed by the wiki as **the Adding-type behaviour with `B_co = B` (original base, no flat)** on all Adding Incarnon attacks listed in §1.7 and by `Braton Incarnon Genesis` § Known Bugs. Multiplying Incarnon attacks (Kunai) do include the flat. Formula and example: §1.4, §1.8 example 3.
- Heavy attacks: "Increased base damage from Incarnon Genesis is not multiplied by the Heavy Attack Multiplier, but is multiplied by the combo multiplier" (`Melee` § Heavy Attack Damage) — a second, separate Incarnon-flat exception (melee).

### 1.12 CO and status DoT

- CO is part of "Modded Base Damage" for procs on melee (`Condition Overload` § Notes: "including proc damage like Slash"; radial procs computed on the initial target).
- Exceptions in the catalogue: Tysis CO "does not affect DoT"; Stinger "does not apply to embedded DoT"; Pox cloud "recalculates on every tick".
- Whether the Adding-type omissions (Eclipse etc.) are also omitted from the DoT seed: UNVERIFIED.

---

## 2. Melee combo, Blood Rush, Weeping Wounds, Gladiator and other combo-scaled bonuses

- Combo multiplier tiers: 2× at 20 hits, +1× per 20 hits, cap 12× at 220 (13× Venka Prime at 240; Dex Nikana 11 hits per tier, cap 11×) (`Melee Combo` § Combo Multiplier).
- **Normal attacks are not multiplied by the combo multiplier.** Heavy attacks get it as a separate multiplier and consume combo (`Melee Combo` § Mechanics; `Melee` § Heavy Attack Damage):
  `Heavy = [B × (1 + A) × (1 + Elemental)] × HeavyTypeMult × Combo` (Paracesis example `222 × 2.2 × 1.9 × 6 × 7 = 38,974.32`).
  Heavy-attack Slash proc total: `[B × (1 + A) × (1 + Faction)] × HeavyTypeMult × Combo × (1 + Faction) × 0.35 × 6` (Reaper Prime example 46,846.8).
- "Multiplier Ability Damage" abilities get combo at 1:0.25 → 1.5×–4× (`Melee Combo` § Mechanics); Ability Combo Counter abilities get the full 2×–12×.
- Combo affects DoT ticks of the inducing hit (`Damage over Time` § DoT Damage Scaling lists Melee Combo Counter and Sniper Combo Counter).
- **Blood Rush** (+40 % per combo tier above 1, relative crit chance; U26 changed it from multiplicative to additive, U30.5 60 → 40 %): `CC = CC_base × [1 + ΣCC_mods + 0.4 × (Combo − 1)] + ΣCC_abs`. Example: 20 % base, True Steel 1.2, combo 4× → `0.2 × (1 + 1.2 + 1.2) = 68 %` (`Blood Rush` § Calculation).
- **Weeping Wounds** (+40 % status chance per tier above 1, relative): `SC = SC_base × [1 + ΣSC_mods + 0.4 × (Combo − 1)]`. Example: 20 %, Voltaic Strike 0.6, combo 4× → `0.2 × (1 + 0.6 + 1.2) = 56 %` (`Weeping Wounds` § Calculation; changed to this form in U26).
- **Gladiator set**: +10 % crit chance per equipped set mod per combo tier above 1 (table: 1 mod 110 % at 12×, 6 mods 660 %) → `0.1 × n_mods × (Combo − 1)` in the relative bucket; stacks with Blood Rush (`Gladiator Set`).
- Combined example (heavy-attack-free): base 25 %, True Steel +120 %, Blood Rush, Gladiator ×3, combo 12×, Arcane Avenger: `0.25 × (1 + 1.2 + 4.4 + 3.3) + 0.45 = 2.475 + 0.45 = 292.5 %`.
- True Steel family and melee riven crit chance are applied **twice on heavy attacks** (`Melee` § Interactions with Heavy Attacks; `Galvanized Steel` U37.0.4: Galvanized Steel heavy = +220 %, a bug that gave +330 % was fixed). Whether Blood Rush/Gladiator use the combo before or after the heavy attack consumes it: UNVERIFIED.
- Secondary Outburst: on swapping to the secondary, consumes melee combo for +20 % crit chance **and** +20 % crit damage per combo tier for 30 s (12× → +240 %); additive with Pistol Gambit / Target Cracker (relative buckets) (`Secondary Outburst` stats and § Notes; reworked U34). Example: 20 % base, Pistol Gambit +120 %, 12× → `0.2 × (1 + 1.2 + 2.4) = 92 %`.
- Exodia Contagion projectile: +0/100/200/300 % after 30 m plus `(combo − 1)/4` (`Exodia Contagion` § Mechanics). Bucket of that bonus: UNVERIFIED.
- Glaive heavy-attack explosion is a heavy attack and uses the combo multiplier (`Glaive` line 58); Argo & Vel / Arum Spinosa / Quassus Prime projectiles do not factor combo in their CO part (§1.7).

## 3. Faction bucket

- Faction mods ×1.30 (×1.55 Primed) are a total damage multiplier, additive with Roar (50 % × Strength; Helminth 30 % × Strength), True Master's Font Damage Blessing (25 %), Paracesis Sentient bonus (≤ 60 %), riven faction stats (base 0.45 → ×1.46–1.56 positive, ×0.55–0.76 negative) (`Faction Damage Bonus`; `Roar`; `Universal Weapon Bonuses` § Faction Damage Bonus; `Riven Mods` lines 60, 1020).
- Applied **again** to status DoT (double dip), a **third** time on Extra-Hit DoT and on Blast detonation-triggered Xata's Whisper (`Faction Damage Bonus` § Mechanics; `Extra Hit` § Notes; `Xata's Whisper` § tips).
- Roar is strength-scaled inside the faction sum: `100 × (1 + 1.65) × (1 + 0.5 × 1.3 + 0.3) = 516.75` with Intensify, Serration and Bane on Grineer (`Roar` info). Roar applies to all ability damage too.
- Corrupted/Narmer variants ignore Grineer/Corpus/Infested mods; Kuva Grineer/Amalgam/Deimos are affected; Infested mods do not affect Techrot (`Faction Damage Bonus` § Mechanics).
- Riven faction vs Bane: whether a riven's faction stat sums with Bane (`1 + 0.55 + 0.5`) or multiplies (`1.55 × 1.5`) is not stated on the Riven or Faction pages → UNVERIFIED (Roar/Damage Blessing are explicitly additive; riven most likely the same internal stat).

## 4. Warframe ability buffs

| Buff | Bucket | Value (100 % Strength) | Notes | Source |
|---|---|---|---|---|
| Roar (Rhino) | faction | +50 % (subsumed +30 %) | double-dips DoT; all weapons and abilities | `Roar` |
| Eclipse (Mirage, Solar) | separate multiplier | +200 % (subsumed +30 %); `× (1 + 2.0 × Str)` | additive only with Adding-type CO; applies **twice** on Exodia Contagion and glaive explosions | `Eclipse` info (edited 2026-10-21 to clarify) |
| Vex Armor Fury (Chroma) | base bucket | up to +275 % × Str | `B × (1 + 1.65 + 2.75 × 1.3)`; once additive with Pressure Point and once multiplicative on Exodia Contagion and glaive explosions | `Vex Armor` info |
| Elemental Ward – Toxin (Chroma) | base bucket ("universal") | +35 % × Str holster damage for 3 s | additive with Vex Armor and Serration; multiplicative with Pressure Point on melee projectiles | `Elemental Ward` lines 57-60, 97; `Exodia Contagion` § Interactions |
| Spectral Scream (Chroma) | (ability) | — | not affected by elemental bonuses such as Nourish | `Universal Weapon Bonuses` table |
| Xata's Whisper (Xaku) | Extra Hit (Void) | 26 % × Str of the finished hit | double-dips faction and body part; status chance = weapon's; triggered by Blast detonations (bug) with triple faction and elemental re-applied | `Xata's Whisper`; `Extra Hit` |
| Toxic Lash (Saryn) | Extra Hit (Toxin, forced proc) | 30 % × Str (×2 melee) | per multishot pellet; does not combine with weapon elements; faction ×2 on the extra hit and ×3 on its DoT; all Toxin mods (even when combined) scale its ticks; Elementalist + Emerald shard scale ticks; does not charge Incarnon | `Toxic Lash`; `Extra Hit` |
| Nourish (Grendel) | elemental bucket (Viral) | +75 % × Str (subsumed +45 %) | "additive with existing elemental and IPS mods"; counts as an already-combined element (does not buff Toxin/Cold DoT) | `Nourish`; `Extra Hit` § Notes |
| Amp (Octavia) | base bucket | +25 % to +200 % × Str by sound level | additive with Serration and Pressure Point; double-dips on Exodia Contagion | `Amp (Ability)` lines 19-23; `Exodia Contagion` |
| Metronome Forte (Octavia) | base bucket (melee) | +30 % × Str | additive with Pressure Point | `Metronome` § Forte |
| Metronome Opera (Octavia) | multishot bucket | +30 % × Str | additive with Split Chamber | `Metronome` § Opera |
| Symphony of Mercy Deathbringer (Jade) | base bucket | +100 % × Str | double-dips on Exodia Contagion | `Symphony of Mercy`; `Universal Weapon Bonuses` |
| Shooting Gallery (Mesa) | base bucket | +25 % × Str | | `Shooting Gallery` |
| Peacemaker (Mesa) innate | base bucket | +150 % × Str (fixed 50 base) | ramp-up ×2.5 is a separate multiplier: `50 × (1 + 1.5 × min(n,20)/20) × (1 + 1.5 × Str + A) × (1 + Elem)` | `Peacemaker` lines 22-35 |
| Exalted Blade, Artemis Bow, most exalted weapons | Strength scales **listed base** (100/125/200/250 × Str etc.) | | so Strength multiplies all mod buckets | `Exalted Blade`; `Artemis Bow` |
| Metamorphosis Day (Equinox) | base bucket | +25 % × Str, decays | | `Universal Weapon Bonuses` |
| Rift Torrent (Limbo) | base bucket | +30 % × Str per enemy | double-dips on Exodia | idem; `Exodia Contagion` |
| Absorb (Nyx) | base bucket | up to +400 % | | `Universal Weapon Bonuses` |
| Garuda passive | separate multiplier | up to +100 % | applies twice (multiplicatively) on Exodia Contagion; omitted from Adding CO | `Universal Weapon Bonuses`; `Exodia Contagion`; `Condition Overload (Mechanic)` |
| Parasitic Link (Nidus) | separate multiplier | +25 % × Str (+25 % Strength multiplier too) | omitted from Adding CO | `Universal Weapon Bonuses` |
| Vauban passive / Overdriver | separate multipliers | +25 % / +25 % × Str | Overdriver omitted from Adding CO | idem |
| Navigator (Ivara) | separate multiplier | ×5 × Str | projectiles only | idem |
| Furious Javelin (Excalibur augment) | separate multiplier (melee) | | does not multiply Adding CO | `Furious Javelin` |
| Duality, Hall of Mirrors modifier | separate multipliers | | omitted from Adding CO | `Condition Overload (Mechanic)` § Math |
| Covenant (Harrow) | absolute crit chance | ≤ 50 % (×4 on weak points, cap 200 %) | | core-formula file § 8 |
| Molecular Prime, Sonar etc. | enemy vulnerability | | Sonar weak spots do not scale procs | `Damage over Time` § DoT Damage Scaling |

Helminth-subsumed Roar and Eclipse use the reduced values above (`Roar` / `Eclipse` § helminth).

## 5. Arcanes

| Arcane | Effect at R5 | Bucket | Source |
|---|---|---|---|
| Arcane Avenger | +45 % crit chance (absolute) | crit chance absolute (added after mods) | `Arcane Avenger` § Notes |
| Arcane Fury | +180 % melee damage (60 % on crit, 18 s) | base bucket (additive with Pressure Point) | `Arcane Fury` § Notes; U31.6 values |
| Arcane Rage | +180 % primary damage (15 % on headshot, 24 s) | base bucket | `Arcane Rage` |
| Arcane Precision | +300 % secondary damage on weak-point hit, 18 s | base bucket | `Arcane Precision` |
| Arcane Strike | +60 % melee attack speed | attack speed (additive with Fury) | `Arcane Strike` |
| Arcane Velocity / Acceleration | +120 % secondary / +90 % primary (non-shotgun) fire rate | fire rate (additive) | `Arcane Velocity`, `Arcane Acceleration` |
| Arcane Arachne | +150 % weapon damage | base bucket on guns; **separate "weapon damage" multiplier** on melee projectiles (Exodia Contagion, glaive throw/explosion, additive with Power Throw); additive on Exodia Epidemic, Tenet Agendus, Tenet Grigori, Wolf Sledge, Ceramic Dagger/Ack & Brunt/Hate Incarnon; not on normal melee | `Arcane Arachne` § Notes |
| Primary / Secondary Merciless | +30 %/stack × 12 = +360 % | base bucket | `Primary Merciless`, `Secondary Merciless` |
| Primary / Secondary Deadhead | +120 %/stack × 3 = +360 %; +30 % headshot multiplier | base bucket; HS multiplier bucket (additive with Target Acquired, sniper zoom, Kuva Chakkhurr; multiplicative with Cernos Prime HS bonus and with weak-point multiplier) | `Primary Deadhead` § Notes; `Primary Acuity` § Notes |
| Primary / Secondary Dexterity | +60 %/stack × 6 = +360 % on melee kill; +7.5 s combo duration | base bucket | `Primary Dexterity`, `Secondary Dexterity` |
| Secondary Outburst | +20 % crit chance and crit damage per consumed combo tier (12× → +240 %) | relative crit chance and relative crit damage | `Secondary Outburst` |
| Secondary Fortifier | ×8 damage to Overguard (×3 at R0), steals Overguard | separate multiplier vs Overguard only | `Secondary Fortifier` |
| Secondary Shiver | +45 %/Cold stack | CO-type (§1) | `Secondary Shiver` |
| Secondary Kinship | +20.04 % crit chance per unique ally buff | relative crit chance (`16 % × (1 + 1.2 + 3 × 0.2)`) | `Secondary Kinship` |
| Shotgun Vendetta | +180 % multishot, +75 % reload, 15 s on kill within 5 m | multishot (additivity with mods not stated → UNVERIFIED) | `Shotgun Vendetta` |
| Arcane Pistoleer | 60 % on pistol weak-point kill: **Infinite Ammo** 12 s (since U44.1.3; was +102 % ammo efficiency); also affects primary and heavy | no damage bucket; removes reloads/ammo consumption in sustained DPS while active | `Arcane Pistoleer` patch history |
| Arcane Crepuscular | +30 % Ability Strength; +3.0× **Final** crit multiplier while invisible | crit damage absolute: `1 + t × (CDM − 1 + 3)` | `Arcane Crepuscular` § Notes |
| Molt Augmented | +0.24 %/kill, max +60 % Ability Strength | ability strength (additive with Intensify) → scales all strength-scaled buffs above | `Molt Augmented` |
| Molt Efficiency | ability duration (additive) | duration | `Molt Efficiency` |
| Arcane Hot Shot | +6 %/Heat-inflicting ability stack, max +300 % crit chance, all weapons | relative crit chance (additive with Point Strike) | `Arcane Hot Shot` |
| Primary Frostbite | up to +120 % crit damage and +90 % multishot (40 Cold procs, any source) | relative crit damage; multishot | `Primary Frostbite` |
| Longbow Sharpshot | +300 % next shot after weak-point hit | separate multiplier (omitted from Adding CO) | `Longbow Sharpshot` |
| Primary Compression | +100 % damage per metre of radius lost (aiming) | separate multiplier on projectile AoE weapons; **additive with Serration on hitscan-AoE weapons** (Ambassador, Mausolon, Braton/Burston Incarnon, Ferrox/Tenet Ferrox, Opticor/Vandal, Trumna/Prime) | `Primary Compression` table ("Multiplies" ×39 rows, "Adds" ×8) |
| Melee Animosity | +42 % crit chance per normal hit to next heavy, max +420 % | relative crit chance | `Melee Animosity` |
| Melee Exposure | +60 % Corrosive per stack (limit +240 %) | elemental bucket | `Melee Exposure` |
| Melee Duplicate | 100 % chance on yellow crit: Extra Hit at 100 % damage, own crit/status rolls, faction once | Extra Hit (special) | `Extra Hit` § Weapon Effects |
| Melee Influence | spreads elemental status damage incl. CO and crit | — | `Melee Influence` |
| Primary Plated Round | damage buff after reloading an empty magazine; partial reload linear, magazine-size log | bucket not stated → UNVERIFIED | `Primary Plated Round` |
| Exodia Contagion | projectile 2× zaw damage (IPS), 5× explosion (Viral), 1× embed DoT (Blast) | CO Multiplying on direct hit; universal buffs separate multiplier | `Exodia Contagion` |

Values were normalised in U31.6 ("only one stat is affected by rank") for Fury, Rage, Strike, Pistoleer and other older arcanes.

## 6. Archon shards (Tauforged = 1.5×)

| Bonus | Value (Taufo.) | Bucket | Source (`Archon Shard`) |
|---|---|---|---|
| Crimson: Melee critical damage | +25 % (+37.5 %) | relative crit damage, additive with Organ Shatter | lines 254-256 |
| Crimson: Primary status chance | +25 % (+37.5 %) | relative status chance, additive with Rifle Aptitude | 259-261 |
| Crimson: Secondary critical chance | +25 % (+37.5 %) | relative crit chance, additive with Pistol Gambit | 264-266 |
| Violet: Primary Electricity | +30 % (+45 %) +10 % (+15 %) per Crimson/Azure/Violet shard | elemental bucket, additive with Stormbringer; "will combine with mods" (combine position UNVERIFIED) | 390-392 |
| Violet: Melee critical damage | +25 % (+37.5 %), doubled if max Energy > 500 | relative crit damage | 394-397 |
| Topaz: Secondary crit chance per Heat-affected kill | +1 % (+1.5 %) per kill, max 50 % (75 %), stacks across shards | relative crit chance | 368-372 |
| Emerald: Toxin status damage | +30 % (+45 %) | status damage, additive with Elementalist | 334-337 |
| Topaz/Violet/Emerald: Ability damage vs Radiation/Electricity/Corrosion-affected | +10 % (+15 %) each, additive across shards | unique ability-damage multiplier, multiplicative with Strength, not on exalted weapons | 344-349, 374-388 |
| Emerald: Corrosion max stacks | +2 (+3) | enemy armor strip | 351-354 |

## 7. Focus (Madurai, Focus 3.0, U31.5)

- **Void Strike**: first ability input consumes all Operator energy → +10 % damage per % energy for 8 s (+1000 % at full), 40 s cooldown. On Warframe weapons it is **additive with the base bucket "like Serration, Condition Overload, or Secondary Merciless"**; additive with Operator/Amp damage bonuses; not on abilities or archguns; counted once (melee damage) on Exodia Contagion (`Focus/Madurai` § Void Strike; `Exodia Contagion` § Interactions). Example: `100 × (1 + 1.65 + 10) = 1265`.
- **Phoenix Talons** (+30 % physical, passive, Warframe weapons incl. exalted, not abilities): adds a flat bonus after physical mods: `Bonus_t = B_total × 0.3 × (B_t / Σ B_physical)` for each physical type `t`; elemental types get nothing. Dual Ichor example: Impact `19 + 122 × 0.3 × 0.254`, Puncture `11 + 122 × 0.3 × 0.146`, Slash `45 + 122 × 0.3 × 0.6`, Toxin 47 unchanged. Physical mods/riven physical stats change the weighting (`Focus/Madurai` § Phoenix Talons). Interaction with the base bucket (does Serration scale it?) not stated → UNVERIFIED.
- Power Transfer (Amp crit damage, casting speed), Sling Strength (+40 % Ability Strength, additive with Intensify), Contamination Wave (operator-damage vulnerability): no weapon damage bucket.
- Old Madurai elemental/Heat bonuses were removed in U31.5.

## 8. Hunter Munitions and Internal Bleeding

- **Hunter Munitions** (Hunter set, primary): 30 % chance on critical hit to force a Slash proc; independent of status chance and damage distribution; works on Artemis Bow (fixed U33). AoE weapons can trigger it twice on a direct hit (direct + AoE). The forced Slash uses modded base damage × the hit's crit tier multiplier × headshot multiplier (not elemental/physical mods) (`Hunter Munitions` § Notes).
- **Internal Bleeding** (rifle): 35 % chance on Impact proc to force Slash, **doubled to 70 % when fire rate < 2.5** ("exactly 2.5 won't count") (`Internal Bleeding` stats and § Notes). The Hunter Munitions page writes the threshold as "> 2.5 → 54.5 %", the Internal Bleeding page as ">= 2.5" — consistent with "exactly 2.5 counts as fast".
- Both cannot produce two Slash procs on one damage instance; they roll independently: `P(forced Slash) = 1 − (1 − 0.3·[crit]) × (1 − p_IB·[Impact proc])`. Wiki check: crit and Impact proc, fast weapon `1 − 0.7 × 0.65 = 54.5 %`; slow weapon `1 − 0.7 × 0.3 = 79 %`. They can stack with a Slash proc from the weapon's own status chance (Hunter Munitions page).
- Bow Fire Rate for this threshold is a hidden stat: 1.0 for most bows, Cinta 1.5, Kuva Bramma and Nataruk 0.667 (`Internal Bleeding` § Notes).
- Per pellet expectation (engine use): `E[forced Slash] = 0.3 × min(CC,1)` for Hunter Munitions alone (one roll per pellet regardless of tier: implied by "on critical hit", not stated) → tier-independence UNVERIFIED.

## 9. Stealth and finisher multipliers

- **Stealth Damage Bonus** on standard melee attacks vs unalerted/blinded/stunned-for-finisher enemies or while cloaked: `SDB = 1 + 0.2 × Rank` (max rank 30 → +700 %; rank-40 weapons `1 + 0.15 × Rank` → +800 %) (`Stealth` § Stealth Damage Bonus; `Finisher` § Stealth Attack).
- **Stealth is additive with the crit multiplier**: `Stealth crit = WeaponDamage × (1 + SDB + t × (CDM − 1))`. Example: R30, CDM 3.0, tier 1: `1 + 7 + 2 = 10×` (not `8 × 3 = 24×`) (`Stealth` § Stealth Damage Bonus).
- Stealth finisher (stealth attack): `Damage × TypeMult × (1 + SDB)`; with crit the `Finisher` page writes `× (1 + SDB + Critical Multiplier)` — differs from the `Stealth` page's `t × (CDM − 1)` → conflict, UNVERIFIED. Finisher damage is converted to Finisher (true) damage; "Elemental and Physical damage mods only apply half of their bonus before being converted" (`Finisher` § Stealth Attack) — formula for "half" not given → UNVERIFIED.
- Finisher damage bonus mods: Finishing Touch (+60 %) and Covert Lethality (+100 %, daggers) add: "+160 % bonus finisher damage" (`Covert Lethality` § Notes) → additive finisher bucket, a separate multiplier.
- Bug: weapons with Valence bonuses deal half the expected finisher damage (Kuva Shildeg example 14,455 expected vs 7,226) (`Finisher` § Notes line 622).
- Exodia Contagion projectile is affected by the stealth multiplier (`Exodia Contagion` § Interactions). Status DoT inherits stealth (`Damage over Time` § DoT Damage Scaling).
- Mercy: `Enemy residual health + 1` Finisher damage (`Finisher` § Mercy).

## 10. Headshot / weak-point crit bonuses (bucket summary only; math in core-formula file § 10)

- Weak-point damage (Acuity +350 %, Seek +75 %, riven Weakpoint Damage ≤ 225 % base) adds to the part multiplier at 1.5× on head weak points, 1.0× elsewhere; multiplicative with HS multiplier bonuses (`Primary Acuity` § Notes: `1.3 × (3 + 1.5 × 3.5) = 10.725`).
- Weak-point crit chance (Acuity +350 %, Worthy Comradery, Cyte-09 passive, riven Weakpoint Crit Chance ≤ 247.5 % base): relative crit chance on weak-point hits, additive with Point Strike (`Primary Acuity`; `Universal Weapon Bonuses`).
- Argon Scope (+135 %), Hydraulic Crosshairs, Galvanized Scope/Crosshairs (+120 % + 40 % × 5 = +320 %, each stack has its own timer; trigger "on weak point kill" since U44.0.2): relative crit chance while aiming, additive with Point Strike (`Argon Scope` § Notes; `Galvanized Scope`).
- Headcrit: 2× crit multiplier on headcrit parts only (core-formula § 10).
- Inspiration changelog mentions it once simulated "mods with critical chance on weak points multiplied their critical chance on certain weapons" and disabled it 2025-03-20; no wiki source for such a multiplicative case was found → treat WP crit chance as additive.

## 11. Riven stats (buckets)

From `Riven Mods` § Base Values (disposition retired: all weapons fixed at 1.2×, line 7):
- Damage / Melee Damage → base bucket. Multishot → multishot. Critical Chance → relative CC (melee: "x2 for Heavy Attacks"); Critical Chance for Slide Attack → relative CC on slides. Critical Damage → relative CDM. Status Chance → relative SC. Status Duration. Status Damage (spliced) → status-damage bucket. Heat/Cold/Electricity/Toxin and spliced combined elements → elemental bucket (riven with two elements: last listed has priority, core-formula § 5). Impact/Puncture/Slash → physical. Faction (Corpus/Grineer/Infested; spliced Orokin/Scaldra/Techrot) → faction bucket as `×(1 + v)`. Finisher Damage → finisher bucket. Melee Damage on Heavy Attack (spliced) → base bucket, heavy attacks only (`Melee` § Melee Damage on Heavy Attack: "additive with Pressure Point"). Slam Attack Damage (spliced) → Slam Damage multiplier (multiplicative with Pressure Point, `Melee` § Slam Damage). Weakpoint Damage / Weakpoint Critical Chance (spliced) → §10.
- Value formula: `v = b × 10 × (r + 1) × 1.2 × L × (0.9 + 0.2u)` in binary32, `L` from the bonus/curse table (e.g. 2 bonus 0 curse 0.99, 3 bonus 1 curse 0.9375 / −0.75) (`Riven Mods` § Attribute Value Formula).
- Multishot curse cannot push multishot below 1; Acuity ignores riven multishot (core-formula § 7).

## 12. Melee projectiles and slams: two damage buckets

- Melee projectiles (glaive throws and explosions, Exodia Contagion) have **two** separate base-type buckets that multiply each other: "Melee Damage" (Pressure Point, Arcane Fury, Condition Overload when additive) and "Weapon Damage" (universal: Arcane Arachne, Vigorous Swap, Holster Amp, Toxin Ward, Power Throw). Buffs that are both (Vex Armor, Deathbringer, Octavia Amp, Rift Torrent, Lethal Levitation) **apply twice** (once in each bucket). Eclipse and Garuda's passive apply twice multiplicatively. Void Strike only once (melee) (`Exodia Contagion` § Interactions; `Power Throw` § Notes; `Vex Armor`; `Eclipse`).
  Example (Exodia direct hit, illustrative): zaw `B = 200`, Pressure Point +120 %, Vex Armor +275 %, Arachne +150 %: `2 × 200 × (1 + 1.2 + 2.75) × (1 + 1.5 + 2.75) = 400 × 4.95 × 5.25 = 10,395`.
- Power Throw: +100 % throw damage, multiplicative with Pressure Point, additive with universal weapon damage (`Power Throw` § Notes).
- Glaive airborne-throw detonations do not get the heavy-attack ×2 crit chance; standing-throw Volatile explosions always do (bug; also Wolf Sledge) (`Glaive (Weapon Type)` lines 44-45).
- Slams: 2× weapon damage (heavy slam 3×), single damage type, linear falloff to 50 % (70 % heavy), entirely radial since Dante Unbound (U36); Slam Damage (Seismic Wave +200 %, Nira's set +150 %/mod) is a separate multiplier vs Pressure Point; CO does not apply to slams (`Melee` § Slam Attack, § Slam Damage; `Condition Overload`).
- Follow Through: n-th target takes `FT^(n−1)` (`Melee` § Follow Through).

## 13. Extra Hits

`Extra Hit` § Damage Calculation:
```
ExtraHit = WeaponHitDamage × ExtraHit% × (1 + Faction)
WeaponHitDamage = B × [1 + Elem + IPS-weighted] × (1 + A) × (1 + Faction) × (crit × body-part × other)
```
- Extra-Hit DoT uses the extra-hit damage as modded base (so elemental bonuses enter the seed) and applies faction again (triple dip) and the weapon's **matching** elemental bonus (Toxic Lash example: `100 × 1.9 × 0.3 = 57`, tick `57 × 1.9 × 0.5`).
- Each multishot pellet triggers its own extra hit; extra hits do not combine elements or shift status weights; do not add melee/sniper combo; a lethal parent hit prevents the extra hit.
- All extra hits except Xata's Whisper have a forced 1× weak-point multiplier on the extra hit itself; Xata's double-dips body part.
- Melee Duplicate is the exception: copies the hit, rolls its own crit/status, faction once.
- Toxic Lash tick from the wiki example `78 × 0.5 = 39`; the Damage/Calculation "+1 accumulator" rule (`(seed + 1) × 0.5 = 39.5`) is stated for "weapon-generated" ticks — whether it applies to extra-hit procs is UNVERIFIED.

---

## Master table

| Bonus | Bucket | Exceptions | Source |
|---|---|---|---|
| Serration / Hornet Strike / Point Blank / Pressure Point / Heavy Caliber / Spoiled Strike etc. | base bucket (additive) | multiplied by CO on Multiplying attacks; separate from "weapon damage" bucket on melee projectiles | `Calculating Bonuses`; `Condition Overload (Mechanic)` |
| Negative base mods (Hollow Point, Anemic Agility…) | base bucket (negative entries) | — | `Calculating Bonuses` § Opportunity Cost |
| Condition Overload (melee) +80 %/status | CO-type: additive with base bucket on most attacks | Multiplying on Corufell heavy, Exodia Contagion, Nepheri, Syam, Tenet Agendus, Tenet Grigori, Verdilac, Exalted Umbra heavy waves; reduced/increased by stance (§1.10); not on slams/radial | `Condition Overload`; `Condition Overload (Mechanic)` |
| Galvanized Aptitude / Savvy (+80 %/status), Galvanized Shot (+120 %/status) | CO-type | Multiplying on most projectile attacks (§1.9); `B_co` rules (§1.7); never on AoE radius | idem; mod pages |
| Secondary Shiver +45 %/Cold stack; Shattering Frost | CO-type | same per-attack behaviour; only CO source for archguns | `Secondary Shiver`; `Decree` |
| Cedo +60 %/status | CO-type | primary fire additive; alt-fire direct hit multiplicative | `Cedo` |
| Incarnon per-status perks (Lato, Latron, Angstrum, Despair, Zylok, Soma, Dual Toxocyst) and low-HP perks (Kunai, Dread) | CO-type | Kunai Incarnon value conflict 100 % vs 200 % | `Condition Overload (Mechanic)` |
| Incarnon "+X Base Damage" | flat add to base before mods | excluded from `B_co` on Adding attacks; included on Kunai Incarnon (Multiplying); not multiplied by heavy-attack type multiplier but by combo | `Condition Overload (Mechanic)`; `Braton Incarnon Genesis`; `Melee` |
| Incarnon "+Y Damage" on multishot pellets (Braton Evo II) | separate multiplier on extra pellets only | — | `Braton Incarnon Genesis` |
| Kuva/Tenet/Coda progenitor | adds to base damage | included in `B_co` (since U31.1.7–U31.5); halves finisher damage (bug) | core-formula § 3.2; `Finisher` |
| Faction mods, Roar, Damage Blessing, riven faction, Paracesis | faction bucket (additive) | ×2 on DoT, ×3 on extra-hit DoT; riven+bane additivity UNVERIFIED | `Faction Damage Bonus`; `Roar` |
| Vex Armor Fury | base bucket | ×2 (both buckets) on melee projectiles | `Vex Armor`; `Exodia Contagion` |
| Eclipse | separate multiplier | additive-equivalent only vs Adding CO; ×2 multiplicative on melee projectiles | `Eclipse` |
| Garuda passive, Parasitic Link, Vauban passive, Overdriver, Navigator, Duality, Hall of Mirrors, Furious Javelin, Extinguished Dragon Key, falloff | separate multipliers | omitted from Adding CO (except Vauban passive, Navigator: not listed → UNVERIFIED) | `Universal Weapon Bonuses`; `Condition Overload (Mechanic)` |
| Octavia Amp | base bucket | ×2 on melee projectiles | `Amp (Ability)` |
| Metronome Forte / Opera | base bucket (melee) / multishot | — | `Metronome` |
| Deathbringer, Shooting Gallery, Metamorphosis, Rift Torrent, Absorb, Provoked, Vigorous Swap, Holster Amp, Lethal Levitation, Toxin Ward | base bucket on guns | separate "weapon damage" bucket on melee projectiles; Vigorous/Holster/Arachne not on normal melee | `Universal Weapon Bonuses`; buff pages |
| Peacemaker innate | base bucket | ramp-up ×2.5 separate | `Peacemaker` |
| Xata's Whisper / Toxic Lash / Resupply / Silken Stride / Demonium / Gastroparesis / Narin passive | Extra Hit | Xata double-dips body part; Toxic Lash ×2 on melee | `Extra Hit` |
| Nourish | elemental (Viral, already combined) | does not buff Toxin/Cold DoT | `Nourish`; `Extra Hit` |
| Arcane Fury / Rage / Precision / Merciless / Dexterity / Deadhead damage | base bucket | — | arcane pages |
| Arcane Arachne | base bucket on guns | separate bucket on Exodia Contagion and glaives; additive on several melee projectiles | `Arcane Arachne` |
| Deadhead HS multiplier | HS multiplier bucket | multiplicative with Cernos Prime HS and WP multiplier | `Primary Deadhead` |
| Arcane Avenger, Cat's Eye, Covenant | crit chance absolute | Cat's Eye not on abilities | core-formula § 8 |
| Blood Rush, Gladiator set, Melee Animosity, Hot Shot, Secondary Outburst/Kinship, Archon CC, Argon/Galvanized Scope | crit chance relative | True Steel / melee riven CC doubled on heavy attacks | §2, §5, §6, §10 |
| Weeping Wounds | status chance relative | — | `Weeping Wounds` |
| Arcane Crepuscular CDM | crit damage absolute ("final") | not on Gyre abilities | `Arcane Crepuscular` |
| Secondary Outburst CDM, Primary Frostbite, Crimson/Violet shard CDM, Galvanized Steel | crit damage relative | — | §5, §6 |
| Elementalist mods, Emerald shard, riven Status Damage, Galvanized Elementalist | status damage | DoT only | `Archon Shard`; `Damage over Time` |
| Galvanized Diffusion/Chamber/Hell | multishot (+230 % total) | — | mod pages |
| Shotgun Vendetta | multishot | additivity UNVERIFIED | `Shotgun Vendetta` |
| Secondary Fortifier | ×8 vs Overguard | Overguard only | `Secondary Fortifier` |
| Longbow Sharpshot | separate multiplier | omitted from Adding CO | `Longbow Sharpshot` |
| Primary Compression | separate multiplier (projectile AoE) | **additive** with Serration on hitscan-AoE weapons | `Primary Compression` |
| Void Strike (Madurai) | base bucket (+1000 %) | counted once on melee projectiles; not archguns | `Focus/Madurai` |
| Phoenix Talons | flat physical add after physical mods | weighting by physical proportions | `Focus/Madurai` |
| Melee combo multiplier | separate multiplier on heavy attacks only | Incarnon flat multiplied by combo but not heavy type mult | `Melee` |
| Heavy-attack type multiplier, Slam multiplier, Slam Damage mods | separate multipliers | — | `Melee` |
| Melee Damage on Heavy Attack (Killing Blow, Melee Assimilation, riven) | base bucket, heavy only | — | `Melee` |
| Stealth Damage Bonus | adds to the crit multiplier term | finisher formula conflict | `Stealth`; `Finisher` |
| Finisher damage (Finishing Touch, Covert Lethality, riven) | finisher bucket (additive) | elemental/physical half-applied | `Finisher`; `Covert Lethality` |
| Hunter Munitions / Internal Bleeding | forced Slash procs | max one Slash per instance from the two | mod pages |
| Sniper shot combo | separate multiplier, also on DoT | — | `Sniper Rifle`; `Damage over Time` |
| Ability Strength (Molt Augmented, Sling Strength, Crepuscular, Intensify) | strength (additive) | multiplies strength-scaled buffs and exalted base damage | buff/arcane pages |

---

## Edge cases

1. CO-type bonuses are additive with the base bucket on most hitscan/melee attacks but a separate multiplier on most projectile attacks; the catalogue (§1.9) must be per-attack data, not a per-weapon flag.
2. Additive CO ignores Eclipse, Garuda passive, Parasitic Link, Overdriver, Duality, Furious Javelin, Hall of Mirrors, Longbow Sharpshot, projectile Primary Compression, Dragon Key and range falloff; beam ramp-up is not ignored.
3. Incarnon flat base damage is excluded from the CO base on Adding attacks (Furis, Lex Prime, Atomos, Bronco Prime, Cestra, Despair, Dual Toxocyst, Lato Vandal, Vasto Prime, Zylok Prime, Braton Prime) but included on Kunai Incarnon.
4. Bows and charge weapons use uncharged damage as CO base (≈50 % effectiveness at full charge); Lanka, Miter, Drakgoon, Evensong, Ballistica family use partial values; Corvas respects charge.
5. Child projectiles (Kulstar, Kuva Bramma, Zarr, Kuva Zarr, Sporelacer bomblets) use the parent projectile's damage as CO base — CO stronger than listed.
6. Hitscan-AoE attacks give CO only to the directly hit target, scaled from the hitscan damage, not multiplied by multishot.
7. Several "should work" attacks get no CO at all (Balefire Charger, Stug, Ferrox embed, Azima turret, Sonicor, Scyotid grab, Coda Bassocyst alt, Proboscis Cernos AoE).
8. Archguns only get CO from Secondary Shiver and Shattering Frost.
9. Noctua and Artemis Bow CO bases ignore Ability Strength; Artemis Bow loses CO entirely with Concentrated Arrow.
10. Exalted Umbra Blade: light waves Adding at 200 %, heavy waves Multiplying.
11. Tenet Grigori disk: melee base mods don't apply, universal weapon damage does and is multiplied by CO.
12. Tenet Detron: CO only 65 % effective on multishot-created projectiles.
13. CO bonus only applies to hits after the status was applied; multishot pellets and gunblade pellets count as sequential hits.
14. Secondary Shiver counts Cold stacks (up to 10), not status types.
15. Stance inputs have CO effectiveness from 25 % to 233 % (Exalted Blade 122–130 % on every tested input).
16. CO is in the modded base of procs (Slash, radial Blast/Electricity/Gas computed on the initial target), but Tysis and Stinger DoT ignore it and Pox recalculates per tick.
17. Melee projectiles have separate melee-damage and weapon-damage buckets; Vex Armor, Deathbringer, Octavia Amp, Rift Torrent, Lethal Levitation count twice; Eclipse and Garuda passive multiply twice.
18. Primary Compression is a multiplier on projectile AoE but additive with Serration on hitscan-AoE weapons.
19. Heavy attacks: combo multiplier and heavy-type multiplier are separate multipliers; Incarnon flat base is multiplied by combo but not by heavy-type multiplier; True Steel and melee riven CC are doubled.
20. Blood Rush, Weeping Wounds and Gladiator use `(combo − 1)` and sit in the relative bucket (pre-U26 they were multiplicative).
21. Stealth Damage Bonus adds to `t × (CDM − 1)` instead of multiplying it.
22. Valence (Kuva/Tenet) melee weapons deal half finisher damage (bug); finishers apply only half of elemental/physical mods.
23. Faction bonuses hit DoT twice, Extra-Hit DoT three times; Xata's Whisper on Blast detonations re-applies faction and elementals.
24. Extra hits have a forced 1× weak-point multiplier except Xata's Whisper; Toxic Lash doubles on melee; Melee Duplicate applies faction once.
25. Internal Bleeding chance doubles below 2.5 fire rate (exactly 2.5 counts as fast; bows use hidden fire rate 1.0/1.5/0.667).
26. Arcane Pistoleer now grants infinite ammo (U44.1.3), changing sustained DPS, not per-hit damage.
27. Phoenix Talons adds physical damage weighted by physical proportions after physical mods, even on weapons whose damage is mostly elemental.
28. Void Strike is additive with base mods and CO (+1000 % at full energy).
29. Kunai Incarnon low-HP perk is +100 % in the source list but "+200 % (3x)" in the catalogue row.
30. Peacemaker strength bonus is additive with Hornet Strike; its ramp-up (×2.5) is a separate multiplier.

## Open questions

1. Do crit, faction and body-part multipliers scale the CO part of an Adding attack? Assumed yes; no page states it.
2. Mechanism behind "Multiplying" projectile behaviour — only speculation (AttackData override) on the Trivia subpage.
3. Stance CO ratio: per hit or averaged over the input's hits?
4. Max unique statuses for CO: 16 (Galvanized pages, no Tau) vs 17 (CO mod and Mechanic pages include Tau).
5. Kunai Incarnon "Swift Conclusion": +100 % or +200 %?
6. Riven faction stat vs Bane mod: additive (assumed) or multiplicative?
7. Shotgun Vendetta multishot and Primary Plated Round damage: which bucket?
8. Phoenix Talons physical flat: before or after the base bucket (Serration)?
9. Stealth finisher crit formula: `1 + SDB + CDM` (Finisher page) vs `1 + SDB + t × (CDM − 1)` (Stealth page); exact "half bonus" rule for elemental/physical mods on finishers.
10. Does the DoT "+1 accumulator" rule apply to extra-hit procs (Toxic Lash tick 39 vs 39.5)?
11. Are Vauban passive and Navigator omitted from the Adding-CO recalculation like Overdriver/Eclipse?
12. Do Blood Rush/Gladiator use the pre-consumption combo on heavy attacks?
13. Hunter Munitions: one 30 % roll per pellet regardless of crit tier?
14. CO interaction with damage quantization (is the CO part quantized with the rest?) — not covered by any page.
15. Untested attacks (all Kuva primaries, most Tenet/Coda weapons, several melee) — behaviour unknown; default to Adding 100 % per wiki instruction.
16. Kulstar cluster row: 75 → 200 is 267 %, the table says 257 % — table arithmetic inconsistency.
17. Whether Adding-type omissions also apply to the CO portion of status DoT seeds.

## Sources used

- wiki.warframe.com (raw wikitext, cached in `research/external/cache/wiki/`): Condition Overload (Mechanic) and its /Testing and /Trivia subpages; Condition Overload; Galvanized Aptitude, Savvy, Shot, Diffusion, Chamber, Hell, Scope, Crosshairs, Acceleration, Steel, Elementalist, Reflex; Calculating Bonuses; Damage/Calculation; Damage over Time; Universal Weapon Bonuses; Faction Damage Bonus; Faction Damage Mods; Riven Mods; Blood Rush; Weeping Wounds; Gladiator Set; Melee Combo; Melee; Stealth; Finisher; Covert Lethality; Roar; Eclipse; Vex Armor; Elemental Ward; Xata's Whisper; Toxic Lash; Nourish; Metronome; Amp (Ability); Symphony of Mercy; Shooting Gallery; Provoked; Vigorous Swap; Holster Amp; Furious Javelin; Peacemaker; Exalted Blade; Artemis Bow; Exalted Weapon; Extra Hit; Exodia Contagion; Power Throw; Glaive; Glaive (Weapon Type); Gunblade; Sniper Rifle; Cedo; Daikyu; Kuva Bramma; Decree; Braton Incarnon Genesis; Incarnon; Archon Shard; Focus/Madurai; Hunter Munitions; Internal Bleeding; Primary Acuity; Argon Scope; arcane pages (Arcane Avenger, Fury, Rage, Precision, Strike, Velocity, Acceleration, Arachne, Hot Shot, Crepuscular, Pistoleer, Molt Augmented, Molt Efficiency, Primary/Secondary Merciless, Deadhead, Dexterity, Secondary Outburst, Fortifier, Shiver, Kinship, Encumber, Shotgun Vendetta, Longbow Sharpshot, Primary Compression, Primary Frostbite, Primary Plated Round, Melee Animosity, Exposure, Crescendo, Influence, Duplicate).
- MediaWiki API revision timestamps for freshness (see header).
- Inspiration guide (`research/external/inspiration/guide.txt` lines 320-335) only to identify which mechanic its "Incarnon + GunCO bug simulation" refers to; no claim above relies on it.
