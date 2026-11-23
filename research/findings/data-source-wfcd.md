Status: draft

# Data source evaluation: WFCD warframe-items

Evaluated 2026-11-23 against tag `v1.1276.51` = commit `7fecf34f9499f99986600725958cb2604ffa1ebe` (2026-11-14, "fix(items): new items"), which was `master` HEAD at the time. README badge says the data tracks game version **44.1.4**.
Downloads cached in `research/external/cache/wfcd/` (category JSON, `i18n/de.json`, npm metadata `npm-items.json`, and a sparse clone of `build/` + `config/` in `repo/src/`).
Every number below was measured with Python over those files. The scripts were throwaway, and the numbers can be reproduced from the cache.

## TL;DR

- **Use it for:** the item catalogue (names, `uniqueName`s, categories, types, MR, polarities, slots, tags, release dates, images, i18n names/descriptions) and for DE-export base stats (`fireRate`, `magazineSize`, `reloadTime`, top-level crit/status, `multishot`, `damagePerShot`). It is MIT-licensed, current (Update 44, Sep 2026, plus hotfixes), and auto-rebuilt several times a week.
- **Do not use it alone for:** per-attack data. The `attacks` array is a **lossy copy of wiki `Module:Weapons/data`**. WFCD drops per-attack `Multishot`, `ForcedProcs`, `PunchThrough`, `BurstCount`/`BurstDelay`, `AmmoCost`, `IncarnonCharges`, `AttackParentIndex`, `ExtraHeadshotDmg`, `ExplosionDelay`, `AmmoMax`, and the float riven disposition. It also rounds `ChargeTime` to 1 decimal. Read the wiki module directly instead.
- **Not usable as is:** mod and arcane effects (`levelStats` is display text only, with DE colour tokens), archon shards (no effect data at all), incarnon evolutions (absent), kitgun/zaw part stats (absent), stance combos (absent), and riven disposition (`omegaAttenuation` is a constant 1.2 placeholder, so `disposition` is 4 for almost every weapon).
- **Enemy.json is stale and uses the wrong damage model.** `resistances` are Damage 2.0 health/armor classes (Cloned Flesh, Ferrite Armor and so on), which Update 36 removed. There is no base level, no overguard, and no damage reduction. Techrot, Scaldra, Murmur, Anarch and other newer enemies are missing. `faction` is set on only 33/638 records.

## 1. Licence, cadence, distribution

| Item | Finding |
|---|---|
| Licence | MIT (`LICENSE`: "Copyright (c) 2017 Kaptard"; `package.json` `"license": "MIT"`). Attribution must be kept. The data is derived from DE's public mobile export and from wiki.warframe.com. The wiki's content licence (CC BY-SA) may apply to the wiki-derived fields (`attacks`, `introduced`, polarities, URLs). See Open questions. |
| Upstream inputs (from `config/endpoints.json` and `build/wikia/scrapers/WeaponScraper.ts`) | `http://content.warframe.com/MobileExport/Manifest/Export{Weapons,Upgrades,Sentinels,Resources,Drones,Customs,Flavour,Keys,Gear,RelicArcane,Warframes,Recipes,Enemies}.json` plus wiki Lua modules `Module:Weapons/data/{archwing,companion,melee,misc,modular,primary,secondary,railjack}` fetched with `?action=edit`. |
| Release cadence (npm `@wfcd/items`, measured from registry `time`) | 1,324 published versions. Monthly counts: 2026-01 24, 02 18, 03 41, 04 37, 05 40, 06 25, 07 38, 08 42, 09 28, 10 36, 11 (to 14th) 5. Latest `1.1276.51` on 2026-11-14T03:35Z. Git tags mirror npm versions (`refs/tags/v1.1276.51` -> `7fecf34`; 7,398 tags total). |
| Distribution options | (a) Raw per-category files: `https://raw.githubusercontent.com/WFCD/warframe-items/<tag-or-sha>/data/json/<Category>.json`. Pinning by SHA works (tested: identical bytes to `master`). (b) npm tarball `https://registry.npmjs.org/@wfcd/items/-/items-1.1276.51.tgz`, 98.3 MB unpacked, 66 files, which includes all locales. (c) `git clone --filter=blob:none` with sparse checkout. GitHub REST API access was blocked in this session, so commit history was read from git and npm instead. |
| Sizes of the needed categories (bytes) | Primary 447,838; Secondary 325,914; Melee 659,959; Arch-Gun 48,541; Arch-Melee 16,536; SentinelWeapons 40,115; Mods 5,769,166; Arcanes 292,975; Enemy 2,799,680; Warframes 354,047; Misc 1,277,448; i18n/de 4,460,988. About 12 MB without i18n. |
| Other files in `data/json` | Archwing, Components (crafting catalogue, new: parents now hold `{uniqueName,itemCount}` refs), Fish, Gear, Glyphs, Honoria, Node, Pets, Quests, Railjack, Relics, Resources, Sentinels, Sigils, Skins, and `i18n/{de,es,fr,it,ja,ko,pl,pt,ru,tc,th,tr,uk,zh}.json`. No `All.json` in the git tree (the README mentions one; it seems to be built only for npm). |

## 2. Weapons

### 2.1 Coverage (measured)

| File | Records | Notes |
|---|---|---|
| Primary.json | 197 | types: Rifle 116, Shotgun 37, Bow 17, Sniper 15, Launcher 11, Pistol 1. All 197 have `attacks`. |
| Secondary.json | 149 | Pistol 122, Dual Pistols 14, Throwing 13. All have `attacks`. |
| Melee.json | 270 | Melee 215, **Zaw Component 45**, and **10 real melee weapons mistyped `"type":"Rifle"`** (Dual Viciss, Ekhein, Mk1-Bo, Mk1-Furax, Paracesis, Prisma Skana, Prova, Prova Vandal, Ruvox, Venato). 244 have `attacks`; the 26 without are zaw handles and links. |
| Arch-Gun.json | 20 | All have attacks. The wiki's 20 "(Atmosphere)" variants are absent. |
| Arch-Melee.json | 8 | All have attacks. |
| SentinelWeapons.json | 24 | `category` is `"Primary"` (sic). `type` "Companion Weapon". The 17 beast claws (Smeeta Claws and others) on the wiki are absent. |
| Misc.json | 36 Exalted Weapons (`exaltedSlot`: Melee/Primary/Secondary/Arch-Gun/Arch-Melee), 28 Kitgun Components, 11 infested kitgun parts typed `"Pistol"`, 24 Amp parts, 45 Incarnon Genesis adapters | Total 682 records across these files carry an `attacks` array, with 1,705 attacks in all. |

Compared with the wiki `Weapons/data` modules cached in `research/external/cache/wiki-modules/raw/`: primary 200 wiki entries, 2 not in WFCD (`Lizzie`, `Vinquibus (Primary)`). Secondary 156/0 missing. Melee 250, 3 missing (`Dark Split-Sword (Dual Swords)`/`(Heavy Blade)` collapse to one WFCD record; `Vinquibus (Melee)` is absent, and the WFCD `Vinquibus` in Primary.json has only `["Normal Attack"]`). Modular: all 12 kitgun chamber "(Primary)/(Secondary)" variants and `Mote Amp` are absent as weapons.

Freshness: the newest items present are from Update 44.0 (2026-09-23: Nunchasa, Steflos Prime, Aksondol, Corufell Prime, Narin, Citrine Prime). Hotfix 43.0.7 (Haalvu), Update 43.0 (Pride, Wrath, War Prime, Afentis Prime), and Update 42.0 (Coda Bubonico, Tenet Quanta, Kuva Ghoulsaw, Enkaus) are also present. So the catalogue is current to within weeks. Per-attack data for brand-new items can lag, though. `Haalvu` (2026-07) still has `"damage": {}` on both attacks.

### 2.2 Record shape (Furis, trimmed; real data)

```json
{
 "name": "Furis", "uniqueName": "/Lotus/Weapons/Tenno/Pistol/AutoPistol",
 "damagePerShot": [3, 14, 2.9999995, 0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
 "totalDamage": 20, "criticalChance": 0.050000001, "criticalMultiplier": 2,
 "procChance": 0.12, "fireRate": 10.000001, "masteryReq": 1,
 "productCategory": "Pistols", "slot": 0, "accuracy": 22.222221,
 "omegaAttenuation": 1.2, "noise": "Alarming", "trigger": "Auto",
 "magazineSize": 35, "reloadTime": 1.4, "multishot": 1, "type": "Pistol",
 "damage": {"total": 20, "impact": 3, "puncture": 14, "slash": 2.9999995, "heat": 0, "...": 0, "true": 0},
 "category": "Secondary", "tradable": false, "wikiAvailable": true,
 "attacks": [
  {"name": "Normal Attack", "speed": 10, "crit_chance": 5, "crit_mult": 2, "status_chance": 12,
   "shot_type": "Hit-Scan", "damage": {"impact": 3, "slash": 3, "puncture": 14}},
  {"name": "Incarnon Form", "speed": 12, "crit_chance": 26, "crit_mult": 3.4, "status_chance": 24,
   "shot_type": "Hit-Scan", "falloff": {"start": 10, "end": 16, "reduction": 0.48},
   "damage": {"heat": 100}}
 ],
 "polarities": ["naramon"], "tags": ["Tenno"], "exilusPolarity": "madurai",
 "wikiaUrl": "https://wiki.warframe.com/w/Furis",
 "introduced": {"name": "Vanilla", "parent": "0.0", "date": "2012-10-25", "...": "..."},
 "disposition": 4, "releaseDate": "2012-10-25", "isPrime": false, "masterable": true,
 "parentUniqueNames": ["/Lotus/Weapons/Tenno/Akimbo/AkimboAutoPistols"]
}
```

### 2.3 Top-level fields (from DE export unless noted)

Presence counts are for Primary.json (n=197); the other files are similar.

| Field | Present | Notes |
|---|---|---|
| `name`, `uniqueName` | 197 | `uniqueName` is the stable key, so join on it. Names are title-cased by WFCD (`"Bane Of Grineer"`, not "Bane of Grineer"). `MK1` is renamed to `Mk1`. |
| `damagePerShot` | 197 | Fixed 20-slot array in `config/damageTypes.json` order: impact, puncture, slash, heat, cold, electricity, toxin, blast, radiation, gas, magnetic, viral, corrosive, void, tau, cinematic, shieldDrain, healthDrain, energyDrain, true. Float32 noise (`2.9999995`). |
| `damage` | 197 | The same numbers as a keyed map plus `total`. **For multi-attack weapons it is the sum or a mix of several attacks**, not one attack. Example: Acceltra Prime top level is impact 44 + puncture 42.4 + slash 10.6, which is rocket impact plus explosion. 84 of the 536 non-part weapons have `attacks[0]` damage that differs from `totalDamage` by more than 2% (Kuva Bramma 187 vs 1026; Ogris 100 vs 700; Paris 160 vs 320). |
| `totalDamage` | 197 | Sum of `damagePerShot`. |
| `criticalChance`, `procChance` | 197 | Fractions (0.05), float32 noise. 17 weapons disagree with `attacks[0]` (Miter 0.5 vs 20%, Kohm 0.083 vs 75%, Nataruk 0.5 vs 30%). |
| `criticalMultiplier` | 197 | Number. Disagrees with `attacks[0].crit_mult` on 32 records, mostly zaw tips with 0 and multi-mode weapons (Epitaph 1.2 vs 2.6; Fulmin 2.2 vs 2.4). |
| `fireRate` | 197 | Shots/s from DE (Sybaris Prime 4.717 while the wiki attack `speed` is 3.33, so DE folds the burst in). |
| `multishot` | 197 | Pellet count (Hek 7). **The attack `damage` is per pellet** (Hek attack sums to 75; the wiki Lua has `Multishot = 7` on the attack). |
| `magazineSize` | 195 | Missing on 2 (battery or no-magazine weapons). |
| `reloadTime` | 197 | Seconds. |
| `trigger` | Primary 197 / Secondary 149 / Melee 0 | Values: Auto, Semi, Charge, Burst, Held, Active, Duplex, Auto Burst. No burst count or burst delay anywhere. |
| `accuracy` | 197 | DE accuracy value. |
| `noise` | 197 | "Alarming" or "Silent". |
| `omegaAttenuation` | 197 | **Placeholder.** Measured: 1.2 on 196/197 Primary, 149/149 Secondary, 244/270 Melee (1.0 on the rest), 0.5 on all Arch-Melee. This is not the real riven disposition. |
| `disposition` | 197 | WFCD derives it from `omegaAttenuation` in buckets (`build/parser.ts` L1059-1068), so it is **4 for 196/197 Primary, all 149 Secondary, all 244 Melee with wiki data, and 1 for all Arch-Melee**. Unusable. The wiki has a float `Disposition` (Hek 1.2). |
| `masteryReq`, `slot`, `productCategory` | 197 | `productCategory`: LongGuns, Pistols, Melee, SpaceGuns, SpaceMelee, SentinelWeapons, SpecialItems (exalted). Zaw and kitgun parts also say "Pistols". |
| `polarities` | 143 | Wiki-derived; absent means no pre-installed polarities. `exilusPolarity` 190, `stancePolarity` (melee) 225. |
| `tags` | 197 | Wiki `Traits`: Tenno, Prime, Grineer, Corpus, Kuva Lich (21), Tenet (16), Technocyte Coda (14), Incarnon (only 8), Vaulted, Zaw (11) and others. |
| `maxLevelCap` | 24 Primary / 14 Secondary / 12 Melee | 40 for Kuva, Tenet and Coda weapons (and Paracesis). The only adversary-weapon signal. |
| `ammo` | **0** | WFCD reads wiki `MaxAmmo`, but the wiki now uses `AmmoMax`, so the field is never populated. |
| Melee only | 224 | `blockingAngle`, `comboDuration`, `followThrough`, `range`, `windUp`, `slamAttack`, `slamRadialDamage`, `slamRadius`, `slideAttack`, `heavyAttackDamage`, `heavySlamAttack`, `heavySlamRadialDamage`, `heavySlamRadius`. These are DE numbers (Nikana Prime: heavyAttackDamage 990, slamAttack 594, slideAttack 396, windUp 0.5, range 2.5). Heavy attack exists only here, never in `attacks`. |
| Crafting/meta | varies | `buildPrice`, `components`, `marketCost`, `bpCost`, `vaulted`, `vaultDate`, `introduced{name,url,aliases,parent,date}`, `releaseDate`, `wikiaUrl`, `wikiaThumbnail`, `imageName`. |

### 2.4 `attacks[]` (from wiki `Module:Weapons/data`, transformed by `build/wikia/transformers/transformWeapon.ts`)

Key frequencies over all 1,705 attacks: name 1705, crit_chance 1705, crit_mult 1705, status_chance 1705, damage 1705, speed 1703, shot_type 1401, falloff 851, shot_speed 388, flight 382, slide 247, charge_time 149, slam 18.

| Key | Type / unit | Notes |
|---|---|---|
| `name` | string | Free text from the wiki `AttackName`. 513 "Normal Attack", 238 "Slam Attack", 238 "Heavy Slam Attack", 63 "Incarnon Form", 37 "Charged Shot", 14 "Incarnon Form AoE", and many weapon-specific names ("Partial Reload Explosion", "Cube (direct hit)"). Mode grouping and parent/child (impact plus explosion) are **not** encoded. The wiki `AttackParentIndex` is dropped. |
| `speed` | number | Fire rate or attack speed for that attack (wiki `FireRate`). Missing on 2 attacks. |
| `crit_chance`, `status_chance` | **percent**, `toFixed(2)` | 5 means 5%, unlike the top-level fraction. |
| `crit_mult` | number, `toFixed(1)` | No wiki value currently has 2 or more decimals, so there is no loss today. |
| `charge_time` | seconds, **`toFixed(1)`** | Lossy. Wiki `ChargeTime` values 0.25, 0.319, 0.33, 0.36, 0.39, 0.49, 0.65, 0.75, 0.995, 1.05, 1.09 and 1.192 exist; Daikyu Prime "Quick Shot" 0.25 becomes `0.3` in WFCD. Charge time drives DPS for bows and charge weapons. |
| `shot_type` | string | Counts: AoE 730, Projectile 359, (missing) 304, Hit-Scan 279, Thrown 26, DoT 5, **"Hitscan" 2** (spelling variant). 39 ranged weapons have at least one attack without it (Hind, Quanta, Trumna, Alternox, and others). |
| `shot_speed`, `flight` | m/s | Same value twice (`flight` only when truthy). |
| `falloff` | `{start, end, reduction}` | Metres and a 0-1 reduction. `reduction` is missing on 4 of 851. For AoE this is radius and centre falloff (Acceltra Prime explosion 0-5 m, 0.5). For hitscan it is range falloff. |
| `damage` | map | Lower-case keys, with only non-zero types present: impact, puncture, slash, blast, electricity, heat, radiation, toxin, viral, corrosive, cold, magnetic, gas, void (2). **No tau or true keys** (the transformer has a hard-coded 14-type list). **6 attacks have `{}`**: Haalvu x2, Simulor and Synoid Simulor "Orb Launch", Shadow Clones (+Prime). Per pellet for shotguns. |
| `slide` | **string** | `"396"` or with element suffix `"448electricity"`, `"318heat"`. Only on `attacks[0]` of melee. |
| `slam` | `{damage: "256.00", radial: {damage: "0.00", radius: 8}}` | Strings. Only 18 records, the old-format ones. Most melee weapons have separate "Slam Attack"/"Heavy Slam Attack" attacks instead. |
| Declared but never emitted | — | The transformer's `pellet`, `duration`, `radius`, `jump`, `wall`, `channeling` appear 0 times. They map to wiki keys that no longer exist. |

**Fields the engine needs that WFCD drops but wiki `Module:Weapons/data` has** (key counts across the primary, secondary and melee Lua, cached copy): `Multishot` 734, `ForcedProcs` 687, `PunchThrough` 687, `AmmoCost` 517, `AttackParentIndex` 187, per-attack `Trigger` 254, `IncarnonCharges` 67, `BurstDelay` 56, `BurstCount` 50, `ExplosionDelay` 23, `EffectDuration` 18, per-attack `Reload` 16, `ExtraHeadshotDmg` 12, `BurstReloadDelay` 7, and weapon-level `AmmoMax`, `Disposition` (float), `IncarnonChargeGain`. Wiki excerpt for comparison (Hek):

```lua
Hek = {
  AmmoMax = 120,
  Attacks = { {
    AttackName = "Normal Attack", CritChance = 0.1, CritMultiplier = 2,
    Damage = { Impact = 11.25, Puncture = 48.75, Slash = 15 },
    Falloff = { EndRange = 20, Reduction = 0.8, StartRange = 10 },
    FireRate = 2.17, Multishot = 7, PunchThrough = 0, ShotType = "Hit-Scan", StatusChance = 0.107 } },
  Disposition = 1.2, Magazine = 4, Reload = 2, Trigger = "Semi-Auto", ...
}
```

WFCD Hek attack: `{"name":"Normal Attack","speed":2.17,"crit_chance":10,"crit_mult":2,"status_chance":10.7,"shot_type":"Hit-Scan","falloff":{"start":10,"end":20,"reduction":0.8},"damage":{"impact":11.25,"slash":15,"puncture":48.75}}`. Multishot and disposition are lost.

### 2.5 Special weapon families

- **Incarnon.** 69 weapons have at least one attack named `Incarnon*` (Furis, Braton family, Laetum, Phenmor, Felarx, Onos, and others). Only 8 carry the `Incarnon` tag (Laetum, Phenmor, Felarx, Innodem, Praedos, Ruvox, Thalys, Onos), so do not use the tag to detect incarnon. The Incarnon Form attack is present for guns. It is **absent for all 14 melee Genesis families** (Ack & Brunt, Anku, Bo, Ceramic Dagger, Destreza, Dual Ichor, Furax, Hate, Magistar, Nami Solo, Obex, Okina, Sibear, Skana) and for the native melee incarnons Innodem, Praedos, Ruvox and Thalys. **Evolution perks are not in WFCD anywhere.** The 45 `"* Incarnon Genesis"` records in Misc.json are bare adapters with a flavour `description`. The wiki `Mods_data` has `Evo*` internal-named entries (for example `EvoRifleDamageOnHeadshots`, `EvoPistolDamageOnNoCritAndNoProcHit`) that look like the evolution perks. They are absent from WFCD Mods.json.
- **Kuva, Tenet, Coda.** Identified by tag (`Kuva Lich` 21, `Tenet` 16, `Technocyte Coda` 14) and `maxLevelCap: 40`. **There is no progenitor or bonus element field.** For example, Kuva Nukor's attack is `{"radiation":21}` only. The user must supply the bonus element and percentage.
- **Zaw parts.** In Melee.json, `type: "Zaw Component"`. Strikes (tips) have one wiki attack, e.g. Kronsh `{"name":"Kronsh Strike","speed":0.85,"crit_chance":18,"crit_mult":2,"status_chance":18,"damage":{"impact":163.8,"slash":70.2}}`. Each tip is duplicated with a `PvPVariant` uniqueName. Top-level stats are zero, with junk `damagePerShot` (`magnetic: 0.50006479`). **Grips and links have no stats at all** (Peye, Vargeet Ii Ruhang: all zero, no `attacks`). One-handed vs two-handed, speed and damage modifiers, and link crit/status bonuses are all missing. Wiki `Module:Weapons/data/modular` or `Module:Modular/data` is needed.
- **Kitguns.** In Misc.json, `type: "Kitgun Component"` (28). The four Solaris chambers (Catchmoon, Gaze, Rattleguts, Tombfinger) have one attack with **empty damage and all-zero stats**. Catchmoon is wiki-matched to `Catchmoon_(Primary)` with `"damage":{}`, `speed 0`, `crit_chance 0`. Infested kitgun parts (Sporelacer, Vermisplicer, Palmaris, Ulnaris, Arcroid, Thymoid) are typed `"Pistol"` and have no attacks. Grips and loaders have no stat modifiers. **Kitgun data is effectively missing.**
- **Exalted weapons.** 36 in Misc.json with `exaltedSlot`. Attacks exist (Exalted Blade, Balefire Charger, Desert Wind and others), but top-level and attack values disagree. Exalted Blade top-level `totalDamage` is 35, while `attacks[0]` sums to 250 (impact 37.5, slash 175, puncture 37.5). Balefire Charger top-level damage is 0. Shadow Clones attack damage is `{}`. Ability-strength scaling is not described.
- **Melee stances.** 78 `Stance Mod` records in Mods.json have name, polarity, `compatName` (e.g. "Nikanas"), `baseDrain` -2 and `description`, but **no combo data** (no hits, multipliers or forced procs) and no `levelStats`. Wiki `Module:Stances/data` exists in the cache.

## 3. Mods (Mods.json)

### 3.1 Coverage

1,809 records. Types: Warframe Mod 457, Primary 226, Melee 173, Secondary 169, Companion 158, Shotgun 120, Focus Way 105 (sic: focus nodes stored as mods), Plexus 82, Stance 78, Arch-Gun 38, Parazon 38, Necramech 28, K-Drive 23, Tektolyst Artifact 21, Arch-Melee 20, Mod Set Mod 19, Archwing 15, Railjack 10, Posture 6, Riven placeholders (15 across 8 types), and others. There are 746 weapon mods (Primary, Secondary, Melee, Shotgun, Arch-Gun, Arch-Melee), of which 744 have `levelStats`.
There are 138 duplicated names. For example, `Bane Of Grineer` appears three times, once per weapon class, with different `uniqueName`s, and `Ammo Drum` appears three times. Always key on `uniqueName`.
Compared with the wiki `Mods_data`: 1,657 wiki names, 246 not found in WFCD by name. Most are internal-named innate or evolution entries (`Evo*`, `*InnateMod`, `*RandomMod`), set headers ("Amar Set") and PvP mods ("Blade Storm", "Corroding Barrage").

### 3.2 Record shape (Galvanized Chamber, real data, ranks trimmed)

```json
{
 "uniqueName": "/Lotus/Upgrades/Mods/Rifle/WeaponFireIterationsSPMod",
 "name": "Galvanized Chamber", "polarity": "madurai", "rarity": "Rare",
 "baseDrain": 6, "fusionLimit": 10, "compatName": "Rifle", "type": "Primary Mod",
 "levelStats": [
  {"stats": ["+7.3% Multishot", "On Kill:\\n+2.7% Multishot for 20s. Stacks up to 5x."]},
  "... 9 more ranks ...",
  {"stats": ["+80% Multishot", "On Kill:\\n+30% Multishot for 20s. Stacks up to 5x."]}
 ],
 "tradable": true, "transmutable": true, "isPrime": false, "masterable": false
}
```

| Field | Present (of 1,809) | Notes |
|---|---|---|
| `uniqueName`, `name`, `type`, `category` | 1809 | `type` gives the slot family. |
| `polarity` | 1790 | madurai 566, naramon 444, zenurik 258, vazarin 250, universal 139, penjaga 94, unairu 33, umbra 5, aura 1. |
| `rarity` | 1790 | Common, Uncommon, Rare, Legendary. |
| `baseDrain` | 1790 | Rank-0 drain. Auras and stances are negative (Steel Charge -4, Blind Justice -2), meaning they add capacity. |
| `fusionLimit` | 1790 | Max rank (Serration 10, Point Strike 3, Condition Overload 5). `levelStats.length == fusionLimit + 1`. |
| `compatName` | 1586 | Weapon mods: Pistol 137, Melee 137, Shotgun 118, Rifle 118, Archgun 38, Archmelee 20, Assault Rifle 15, Sniper 14, Thrown Melee 13, **PRIMARY 10** (set mods, upper-case), Bow 10, Tome 8, `Rifle (No Aoe)`, `Pistol (No Aoe)`, `Polearms`, `Whips`, `Daggers`, and **96 weapon mods whose `compatName` is a specific weapon name** (weapon augments and Amalgam mods). Not machine-ready: class names do not map 1:1 to weapon `type` (e.g. "Assault Rifle", "Thrown Melee", "Tome"). Wiki `Mods_itemCompatibility_data` is the likely fix. |
| `isAugment` | 457 | **Set only on Warframe mods.** 0 weapon mods are flagged, although the 96 weapon-specific mods above include syndicate weapon augments. |
| `isExilus` | 37 | Warframe exilus only. Weapon exilus is `isUtility` (195, mixed frame and weapon). |
| `modSet`, `modSetValues`, `numUpgradesInSet` | 72 / 5 / 19 | Set membership (e.g. Vigilante, Hunter). `modSetValues` is numeric only for 5 Umbral and Sacrificial mods (`[0.25, 0.5]`). Set bonus text lives in 19 `"Mod Set Mod"` records' `stats` string array (e.g. `Huntersetmod`). |
| `upgradeEntries` | 17 | Riven placeholders only (Rifle, Pistol, Melee and others) and "Unfused Artifact". **Numeric** riven stat pool (see 3.4). |
| `levelStats` | 1625 | `[{stats: [string, ...]}, ...]` per rank. **Text only.** |
| `drops`, `wikiaUrl`, `introduced`, `transmutable` | various | Not needed by the engine. |

### 3.3 `levelStats` sample (30+ DPS-relevant mods; rank 0 and max rank, verbatim)

| Mod | type / compat / drain / max rank | rank 0 | max rank |
|---|---|---|---|
| Serration | Primary / Rifle / 4 / 10 | `+15% Damage` | `+165% Damage` |
| Point Strike | Primary / Rifle / 2 / 3 | `+15% Critical Chance` | `+60% Critical Chance` |
| Vital Sense | Primary / Rifle / 2 / 3 | `+15% Critical Damage` | `+60% Critical Damage` |
| Split Chamber | Primary / Rifle / 4 / 3 | `+5% Multishot` | `+20% Multishot` |
| Hornet Strike | Secondary / Pistol / 2 / 3 | `+10% Damage` | `+40% Damage` |
| Galvanized Chamber | Primary / Rifle / 6 / 10 | `+7.3% Multishot`, `On Kill:\n+2.7% Multishot for 20s. Stacks up to 5x.` | `+80% Multishot`, `On Kill:\n+30% Multishot for 20s. Stacks up to 5x.` |
| Galvanized Aptitude | Primary / Rifle / 2 / 10 | `+7.3% Status Chance`, `On Kill:\n+3.6% Direct Damage per Status Type affecting the target ... Stacks up to 2x.` | `+80% Status Chance`, `... +40% Direct Damage per Status Type ...` |
| Condition Overload | Melee / Melee / 10 / 5 | `+13.3% Melee Damage per Status Type affecting the target` | `+80% Melee Damage per Status Type affecting the target` |
| Primed Pressure Point | Melee / Melee / 4 / 10 | `+15% Melee Damage` | `+165% Melee Damage` |
| Blood Rush | Melee / Melee / 4 / 10 | `+3.6% Critical Chance stacks with Combo Multiplier` | `+40% Critical Chance stacks with Combo Multiplier` |
| Weeping Wounds | Melee / Melee / 4 / 5 | `+6.7% Status Chance per Combo Multiplier` | `+40% Status Chance per Combo Multiplier` |
| Berserker Fury | Melee / Melee / 4 / 3 | `On Melee Kill:\n+5.8% Attack Speed for 10s. Stacks up to 2x.` | `... +23.3% Attack Speed ...` |
| Hellfire | Primary / Rifle / 4 / 3 | `+10% <DT_FIRE_COLOR>Heat` | `+40% <DT_FIRE_COLOR>Heat` |
| Malignant Force | Primary / Rifle / 4 / 3 | `+15% <DT_POISON_COLOR>Toxin`, `+15% Status Chance` | `+60% ...Toxin`, `+60% Status Chance` |
| Primed Cryo Rounds | Primary / Rifle / 6 / 10 | `+15% <DT_FREEZE_COLOR>Cold` | `+165% <DT_FREEZE_COLOR>Cold` |
| Critical Delay | Primary / Rifle / 4 / 5 | `+33.3% Critical Chance`, `-3.3% Fire Rate (x2 for Bows)` | `+200% Critical Chance`, `-20% Fire Rate (x2 for Bows)` |
| Vigilante Armaments | Primary / PRIMARY / 4 / 5 | `+10% Multishot` | `+60% Multishot` |
| Hunter Munitions | Primary / PRIMARY / 4 / 5 | `+5% chance to apply <DT_SLASH_COLOR> on Critical` | `+30% chance to apply <DT_SLASH_COLOR> on Critical` (the word "Slash" is missing; only the token remains) |
| Internal Bleeding | Primary / Rifle / 10 / 5 | ` <DT_IMPACT_COLOR>Impact Status Effects have 10% chance to apply a <DT_SLASH_COLOR>Slash Status Effect (x2 when Fire Rate is below 2.5)` | `... 35% chance ...` |
| Heavy Caliber | Primary / Rifle / 6 / 10 | `+15% Damage`, `-5% Accuracy` | `+165% Damage`, `-55% Accuracy` |
| Amalgam Serration | Primary / Rifle / 6 / 10 | `+14% Damage`, `+2% Sprint Speed` | `+155% Damage`, `+25% Sprint Speed` |
| Primed Shred | Primary / Rifle / 6 / 10 | `+5% Fire Rate (x2 for Bows)`, `+0.2 Punch Through` | `+55% Fire Rate (x2 for Bows)`, `+2.2 Punch Through` |
| Shotgun Savvy | Shotgun / Shotgun / 2 / 3 | `+3% Status Chance` | `+12% Status Chance` |
| Bladed Rounds | Primary / Rifle / 4 / 5 | `On Kill:\n+20% Critical Damage when Aiming for 2s` | `On Kill:\n+120% Critical Damage when Aiming for 9s` |
| Argon Scope | Primary / Rifle / 2 / 5 | `On Weak Point Hit:\n+22.5% Critical Chance when Aiming for 2s` | `... +135% ... for 9s` |
| Galvanized Scope | Primary / Rifle / 2 / 10 | `On Weak Point Hit:\n+10.9% Critical Chance when Aiming for 12s\nOn Weak Point Kill:\n+3.6% ... Stacks up to 5x.` | `... +120% ... +40% ...` |
| Primed Fever Strike | Melee / Melee / 6 / 10 | `+15% <DT_POISON_COLOR>Toxin` | `+165% <DT_POISON_COLOR>Toxin` |
| Rifle Elementalist | Primary / Rifle / 4 / 5 | `+15% Status Damage`, `+0.1 Punch Through` | `+90% Status Damage`, `+0.6 Punch Through` |
| Spoiled Strike | Melee / Melee / 4 / 3 | `+25% Melee Damage`, `-5% Attack Speed` | `+100% Melee Damage`, `-20% Attack Speed` |
| Sacrificial Steel | Melee / Melee / 6 / 10 (umbra) | `+20% Critical Chance (x2 for Heavy Attacks)`, `x1.03 Damage to Sentients` | `+220% ... (x2 for Heavy Attacks)`, `x1.33 Damage to Sentients` |
| Organ Shatter | Melee / Melee / 4 / 3 | `+15% Critical Damage` | `+60% Critical Damage` |
| Serrated Edges | Melee / Melee / 2 / 3 | `5% of Damage converted into <DT_PUNCTURE_COLOR>Puncture` | `20% of Damage converted into ...Puncture` |
| Primed Target Cracker | Secondary / Pistol / 4 / 10 | `+10% Critical Damage` | `+110% Critical Damage` |
| Lethal Torrent | Secondary / Pistol / 6 / 5 | `+10% Fire Rate`, `+10% Multishot` | `+60% Fire Rate`, `+60% Multishot` |
| Vile Acceleration | Primary / Rifle / 4 / 5 | `+15% Fire Rate (x2 for Bows)`, `-2.5% Damage` | `+90% Fire Rate (x2 for Bows)`, `-15% Damage` |
| Primed Ravage | Shotgun / Shotgun / 4 / 10 | `+10% Critical Damage` | `+110% Critical Damage` |
| Covert Lethality | Melee / Daggers / 10 / 3 | `+4 Initial Combo`, `+25% Finisher Damage` | `+16 Initial Combo`, `+100% Finisher Damage` |
| Steel Charge | Warframe / AURA / -4 / 5 | `Squad receives +10% Melee Damage` | `Squad receives +60% Melee Damage` |
| Primed Bane Of Grineer | Primary / Rifle / 4 / 10 (name casing as stored) | `x1.05 Damage to Grineer` | `x1.55 Damage to Grineer` |

### 3.4 Turning `levelStats` into machine effects

Measured over the 744 weapon mods with `levelStats`: **512 (69%) have only "simple" max-rank lines**, matching `^([+-]|x)?N%? (<DT_*>)?Words$`. The other 232 (31%) have conditional or prose lines. The most common normalised line patterns, with digits replaced by N:

`+N% Status Chance` 58, `+N% Magazine Capacity` 25, `+N Punch Through` 23, `+N% Critical Damage` 23, `+N% Damage` 23, `+N% <DT_FIRE_COLOR>Heat` 23 (and the same for Electricity, Cold, Puncture, Slash, Toxin, Impact, around 20 each), `+N% Multishot` 20, `+N% Reload Speed` 19, `+N% Critical Chance` 17, `+N% Fire Rate` 16, `xN Damage to Corpus/Grineer/Infested` 13 each, `+N% Melee Damage` 14, `+N% Fire Rate (xN for Bows)` 7, `+N% Critical Chance (xN for Heavy Attacks)` 6, `N% of Damage converted into <DT_..>X` (IPS conversion mods, 4 each), `On Kill:\n+N% ... for Ns. Stacks up to Nx.`, and `On Weak Point Hit:\n...`.

A workable approach:
1. Use only the **max-rank** line and derive lower ranks as `value * (rank+1)/(fusionLimit+1)`. Lower-rank strings are rounded to 1 decimal (`+7.3%` for 80/11 = 7.27), so parsing every rank adds rounding error. Verify the linear rule per mod. Some mods do not scale linearly (UNVERIFIED which ones; check against wiki `Mods_data`).
2. Map the stat phrase through a hand-written table (`"Damage"` -> base damage, `"Melee Damage"` -> base damage on melee, `<DT_FIRE_COLOR>Heat` -> heat element, `"Critical Damage"` -> crit multiplier, `"Status Damage"` -> status-damage, `xN Damage to Grineer` -> faction multiplier, and so on). DE tokens: `DT_IMPACT/PUNCTURE/SLASH/FIRE/FREEZE/ELECTRICITY/POISON/RADIATION/MAGNETIC..._COLOR` plus `<DT_MAGNETIC>`-style tokens without `_COLOR`.
3. Treat `On X:` prefixes, `Stacks up to Nx`, `per Status Type`, `per Combo Multiplier`, `when Aiming`, `(x2 for Bows)` and `(x2 for Heavy Attacks)` as **conditions or modifiers that need hand-authored semantics**. Condition Overload's "per Status Type" and Galvanized Aptitude's "Direct Damage per Status Type" are exactly the edge cases the project cares about, and text cannot say whether they are additive with base damage or a separate multiplier.
4. Pitfalls: the leading space in Internal Bleeding; literal `\\n` (escaped backslash-n inside the JSON string, not a real newline); the missing element word in Hunter Munitions; set bonuses that live in separate `*setmod` records; and duplicated stat lines in arcanes (below).

Conclusion: `levelStats` can seed an effects table semi-automatically (about 70% of weapon mods). An authored override file is still required, and the wiki `Mods_data` / `Upgrades_data` should be checked first for structured numbers.

Riven pool (numeric, from `upgradeEntries` on the "Rifle Riven Mod" placeholder; 38 stat tags):

```json
{"tag": "WeaponCritChanceMod", "prefixTag": "crita", "suffixTag": "cron",
 "upgradeValues": [{"value": 0.016666001, "locTag": "|val|% Critical Chance"}]}
```

These are DE per-unit base values (tags include WeaponDamageAmountMod, WeaponFireIterationsMod, WeaponStatusDamage, WeaponFactionDamageTechrot, WeaponFactionDamageScaldra, and others). Without real per-weapon disposition they are of limited use for riven ranges.

## 4. Arcanes (Arcanes.json)

172 records. Types: Warframe Arcane 68, Operator 22, Secondary 18, Primary 14, Amp 13, Melee 12, Zaw 8, Kitgun 8, generic "Arcane" 7, Bow 1, Shotgun 1. `levelStats` length: 6 ranks for 142, 4 for 26, 0 for 4. No `compatName`, `polarity` or drain. Newest arcanes present: Update 43.0 (Arcane Sculptor, Primary Compression, Melee Assimilation, Secondary Cryogenic, 2026-06-17). Compared with wiki `Arcane_data`: 166 wiki names, 5 missing (all `Zid-An *`).

```json
{"uniqueName": "/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamageOnKill",
 "name": "Primary Merciless", "rarity": "Rare", "type": "Primary Arcane",
 "levelStats": [
  {"stats": ["On Kill:\\n+5% Damage for 4s. Stacks up to 12x."]},
  "... ranks 1-4 ...",
  {"stats": ["On Kill:\\n+30% Damage for 4s. Stacks up to 12x.\\n+30% Reload Speed", "+30% Reload Speed"]}]}
```

The text has the same problems as mods, plus a **duplicated line at max rank**: the rank-5 bonus appears both appended with `\n` and as its own entry (Primary Merciless, Primary Deadhead, and Arcane Avenger's `+1 Arcane Revive`). More examples: `Melee Duplicate`: `On Base Critical Hits: 100% chance for your attack to strike a second time`; `Cascadia Flare`: `On <DT_FIRE_COLOR>Heat Status Effect:\n+12% Damage for 10s. Stacks up to 480%.`; `Secondary Outburst`: one prose sentence. All arcane effects need hand-authored semantics.

## 5. Archon shards

Not modelled. Misc.json has 15 shard items (Amber, Azure, Crimson, Emerald, Topaz and Violet, each with a Tauforged variant, plus two generic "Archon Shard" records). All are `type: "Misc"` with only a flavour description ("A source of anti-entropic radiation..."). There are no effect options or values, so archon shard effects must come from the wiki (`Archon_Shard` page) or be authored by hand.

## 6. Enemies (Enemy.json)

638 records. Types: Grineer 236, Corpus 212, Infestation 53, Orokin 36, Neutral 25, Sentient 16, Predator 12, Stalker 8, and some odd ones (Rifle 14, Shotgun 10, Melee 2, which come from WFCD's type heuristic). There are 155 duplicated names. The `faction` field is present on only 33 records (when present, WFCD often moved it into `type`).

```json
{"uniqueName": "/Lotus/Types/Enemies/Grineer/AIWeek/Avatars/HeavyFemaleGrineerAvatarLeader",
 "name": "Heavy Gunner", "description": "High damage minigun",
 "health": 300, "shield": 0, "armor": 500, "regionBits": 461417, "type": "Grineer",
 "resistances": [
  {"amount": 0, "type": "Shield", "affectors": [{"element": "None", "modifier": 0}]},
  {"amount": 500, "type": "Ferrite Armor", "affectors": [{"element": "Puncture", "modifier": 0.75},
     {"element": "Toxin", "modifier": 0.5}, {"element": "Corrosive", "modifier": 0},
     {"element": "Slash", "modifier": -0.5}, {"element": "Blast", "modifier": -0.5}]},
  {"amount": 300, "type": "Cloned Flesh", "affectors": [{"element": "Slash", "modifier": 0.5}, "..."]}]}
```

| Field | Notes |
|---|---|
| `health`, `shield`, `armor` | Base values, present on all 638. No level is attached. |
| `resistances` | Always 3 entries (shield, armor, health). **These are Damage 2.0 classes** (Cloned Flesh, Ferrite Armor, Alloy Armor, Fossilized, Infested, Flesh, Shield). The wiki (`cache/wiki/Damage.wiki` L14) states they were replaced by one faction-based modifier set ("all Grineer are now exclusively vulnerable to Impact and Corrosive"). The `affectors` modifiers are stale and **must not** be used. |
| Base level | **Absent.** |
| Overguard, damage reduction, eximus, proc immunity | **Absent.** |
| `regionBits`, `drops`, `patchlogs` | Not useful to the engine. |

Staleness: 0 records for Techrot, Scaldra, Murmur/Fragment, Anarch, Hollvanian, Archon, Necramech, Kuva Lich or Sister (searched by name and uniqueName). The wiki cache has dedicated modules for most of these (`Enemies_data_techrot.lua`, `..._scaldra.lua`, `..._themurmur.lua`, `..._anarchs.lua`, `..._narmer.lua`).
Cross-check with the inspiration sample `research/external/inspiration/samples/enemies-min.json` (498 entries, 496 unique names): 272 names match WFCD. 219 of those have identical health, shield and armor; **53 differ**. Most of the 53 are bosses and specials: Eidolon Teralyst 50000/50000/125 vs WFCD 15000/2500/200; Juggernaut 900/0/200 vs 3500/0/200; Acolytes Angst/Malice/Mania 5500/2500/50 vs 350/200/0; Lich and Sister-type names (Dru Pesfor, Azoth) 1000/2500/50 vs generic 150/100/200. Conclusion: **use the wiki Enemies modules, not WFCD, for enemies.**

## 7. Warframes.json and i18n

- Warframes.json: 123 records with `health`, `shield`, `armor`, `power`, `sprintSpeed`, `abilities` (name and description text), `polarities`, `aura`, and `exalted` (35, linking exalted weapons). No ability numbers or rank scaling, so ability buffs (Roar and others) must be authored.
- i18n: `data/json/i18n/<locale>.json`, a flat dict keyed by `uniqueName` (de: 19,511 keys, 4.46 MB). Each value holds translated `name`, `description`, `trigger` and, for mods, `levelStats`:
  `"/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod": {"name": "Einkerbung", "levelStats": [{"stats": ["+15 % Schaden"]}, ...]}`
  Locales: de, es, fr, it, ja, ko, pl, pt, ru, tc, th, tr, uk, zh. This fits the project's "i18n as data" decision for item names and descriptions.

## 8. Rust build-tool notes

- Fetch: `reqwest` (blocking) or `ureq` to get `https://raw.githubusercontent.com/WFCD/warframe-items/v1.1276.51/data/json/{Primary,Secondary,Melee,Arch-Gun,Arch-Melee,SentinelWeapons,Misc,Mods,Arcanes,Warframes}.json`. Pin a tag in a config file and record the SHA and SHA-256 of each file in the bundled output. Raw URLs accept tags and SHAs. Avoid the 98 MB npm tarball unless all locales are wanted.
- Parse with `serde_json` into `#[serde(rename_all = "camelCase")]` structs, with `Option<T>` and `#[serde(default)]` on almost everything (see the presence counts above). Gotchas:
  - attack keys are snake_case (`crit_chance`) while top-level keys are camelCase;
  - the damage map has a key named `true`, which needs `#[serde(rename = "true")]`;
  - `slide` is a string, sometimes with an element suffix;
  - `slam.damage` is a string;
  - `flight` may be absent;
  - `shot_type` comes in both "Hit-Scan" and "Hitscan" spellings;
  - top-level crit and status are fractions while attack values are percents;
  - float32 noise (`10.000001`), so round to a sensible precision when building the bundle.
- Join WFCD records to wiki `Module:Weapons/data` by `uniqueName` (wiki `InternalName`), and take per-attack mechanics from the wiki.

## Open questions

- Content licence for the wiki-derived fields: MIT covers WFCD's code and compilation, but `attacks`, polarities and the like come from wiki.warframe.com, whose licence (CC BY-SA?) was not verified here.
- Whether `omegaAttenuation` is broken in DE's public export or in WFCD's parsing (it is 1.2 almost everywhere). Not determined without fetching DE's export.
- Which mods scale non-linearly per rank, i.e. where `max * (r+1)/(max+1)` is wrong. This needs cross-checking against wiki `Mods_data`.
- Whether `totalDamage` and `damage` for charge and multi-attack weapons follow a consistent DE rule (sum of all attacks? charged only?). Observed inconsistently (Paris 320 vs attack 160; Acceltra Prime sums impact and explosion).
- Whether the `Evo*` entries in wiki `Mods_data` are the complete incarnon evolution set with numeric effects. To be confirmed by the wiki-modules evaluation.
- What "Lizzie" (a wiki primary absent from WFCD) is, and whether Vinquibus' melee mode exists in any open source besides the wiki.
