Status: draft

# Data source evaluation: DE Public Export, plus the abandoned warframe-connector

Evaluated 2026-11-23. All downloads are cached in `research/external/cache/public-export/`:
`index_en.txt.lzma` (raw), `index_en.txt` / `index_en.lf.txt` (decompressed), `manifests/*.json` (all 16 manifests),
`manifest_headers.txt` (HTTP headers of each manifest), `worldstate/warframestat_pc.json` (live world state, used for the build label).
Every count below was measured with Python over those files.

## TL;DR

- **The Public Export is current.** Live `buildLabel` is `2026.10.28.09.54` (Hotfix 44.1.4, 2026-10-27); the index was last modified 2026-10-28 17:14 GMT. WFCD's base stats are a byte-for-byte copy of it (0 stat differences over 668 shared weapons).
- **Use it for:** the canonical `uniqueName` key of every item, plus top-level weapon stats (20-slot damage vector, crit, crit multiplier, status, fire rate, multishot, magazine, reload, trigger, MR, riven disposition) and melee heavy/slam/slide totals. It also gives the mod catalogue (polarity, rarity, drain, max rank, coarse `compatName`, mod sets), arcanes, riven stat base values, and warframe base stats.
- **It cannot be the only source.** It has **one attack per weapon** and sometimes sums two attacks into one vector (Kuva Bramma: direct 187 Impact + radial 839 Blast). It has no charge time, burst count, falloff, AoE radius, punch-through, forced procs or alt-fire. It has no incarnon forms or evolutions, and no kitgun or zaw part stats (all 182 part records are zeroed). Mod and arcane effects are **display text only**: 10,431 `levelStats` lines with `<DT_…>` tokens and rounded numbers. There are **no enemies** and no archon shard effects.
- **Riven data is the one machine-usable effect table:** 280 `upgradeEntries` give each riven stat's base value per weapon class. For example Rifle Crit Chance is `0.016666` and ×90 gives 150%, which matches the wiki. Riven disposition is retired: `omegaAttenuation` is 1.2 for 649/659 damage-dealing weapons, and the wiki agrees.
- **No licence is stated.** See Open questions.
- **warframe-connector:** it never used the Public Export. The parts we can reuse are its `PoliteClient` pattern (per-host spacing, single-flight, TTL cache, 429/503/509 back-off with Retry-After), its name resolver, and the "wire shapes stay in boundary modules" rule. Its data sources (warframe.market, DE riven prices, usage stats) do not apply to us.

---

# Part 1: DE Public Export

## 1.1 Access: index, manifests and fetching

| Item | Finding |
|---|---|
| Index URL | `https://content.warframe.com/PublicExport/index_<lang>.txt.lzma`. `index_en` and `index_de` both return 200. WFCD fetches 15 locales the same way. |
| Index format | Legacy **LZMA "alone"** (`.lzma`, not `.xz`). Header bytes are `5d 00 00 00 01 1e 03 00 00 00 00 00 00`: props 0x5d, 16 MiB dictionary, 8-byte uncompressed size = 798. The file is 496 bytes. It decompresses to 16 lines, each `<file>!<hash>`, with **CRLF line endings**. |
| Index headers | `Cache-Control: no-cache`, `Last-Modified: Wed, 28 Oct 2026 17:14:19 GMT`, `ETag`. Conditional GET (If-None-Match) is the cheap way to check for updates. |
| Manifest URL | `https://content.warframe.com/PublicExport/Manifest/<file>!<hash>`, with the index line used **verbatim**. Pitfalls I hit: (1) a trailing `\r` on the line gives 404; (2) percent-encoding `+` as `%2B` also gives 404. Pass the line through unchanged. A request without the hash (`.../ExportWeapons_en.json`) is also 404. |
| Manifest caching | The URLs are content-addressed. Headers say `Cache-Control: public, max-age≈1 year`. Cache them forever, keyed by hash. |
| `origin.warframe.com` | WFCD prefers `https://origin.warframe.com/PublicExport/index_*.txt.lzma` and logs "content.warframe.com ... might not be up-to-date" when it falls back (`wfcd/repo/src/build/scraper.ts:68-91`). Origin was blocked by this sandbox's proxy (403 at CONNECT), so I could not compare them. Content matched the live build on this date (see 1.7). |
| MobileExport | WFCD's `config/endpoints.json` still lists `http://content.warframe.com/MobileExport/Manifest/Export*.json`, including `ExportEnemies.json`. All of them now return **404**. WFCD's real build path uses `PublicExport` (`scraper.ts:151`). |
| JSON hygiene | All 16 files are single-line JSON with **0 raw control characters** (measured). Python's strict `json.load` parses all of them. WFCD still runs a `sanitize()` that strips raw `\r`, `\n` and `\t` (`build/network.ts`). Older exports needed this, so keep a lenient fallback. Strings contain escaped `\r\n`. |
| Float noise | Values are float32 printed as-is: `0.050000001`, `10.000001`, `0.19999999`. Round on import (for example to 6 significant digits) or compare with a tolerance. |

Manifest list with sizes and top-level keys (measured):

| File | Bytes | Top-level arrays (record counts) | Last-Modified |
|---|---|---|---|
| ExportWeapons_en.json | 613,322 | ExportWeapons 841, ExportRailjackWeapons 143 | 2026-10-22 |
| ExportUpgrades_en.json | 3,006,834 | ExportUpgrades 1,603, ExportModSet 19, ExportAvionics 82, ExportFocusUpgrades 105 | 2026-10-22 |
| ExportRelicArcane_en.json | 3,359,538 | ExportRelicArcane 3,373 (3,201 relics + 172 arcanes) | 2026-10-22 |
| ExportWarframes_en.json | 236,701 | ExportWarframes 127, ExportAbilities 13 (Helminth) | 2026-10-15 |
| ExportResources_en.json | 1,009,335 | ExportResources 3,552 | 2026-10-27 |
| ExportSentinels_en.json | 11,675 | ExportSentinels 34 | 2025-03-18 |
| ExportCustoms_en.json | 985,449 | ExportCustoms 4,836 (cosmetics only) | 2026-10-07 |
| ExportGear_en.json | 54,737 | ExportGear 180 | 2026-09-21 |
| ExportRecipes_en.json | 1,307,697 | ExportRecipes 1,885 | 2026-09-21 |
| ExportManifest.json | 3,838,380 | Manifest 20,154 (`uniqueName` → `textureLocation` image path) | 2026-10-27 |
| ExportSortieRewards_en.json | 79,718 | SortieRewards 17, Nightwave 3, Railjack 1, Intrinsics 5, Other 46 | 2026-08-12 |
| ExportFlavour / FusionBundles / Keys / Regions / Drones | small | not relevant to damage | — |

There is **no `ExportArcanes`**; arcanes live in `ExportRelicArcane`. There is **no `ExportEnemies`** in the index, and the old MobileExport URL is 404.

## 1.2 ExportWeapons

Real record (Furis, untrimmed apart from `description`):

```json
{
 "name": "Furis",
 "uniqueName": "/Lotus/Weapons/Tenno/Pistol/AutoPistol",
 "codexSecret": false,
 "damagePerShot": [3, 14, 2.9999995, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
 "totalDamage": 20,
 "criticalChance": 0.050000001,
 "criticalMultiplier": 2,
 "procChance": 0.12,
 "fireRate": 10.000001,
 "masteryReq": 1,
 "productCategory": "Pistols",
 "slot": 0,
 "accuracy": 22.222221,
 "omegaAttenuation": 1.2,
 "noise": "ALARMING",
 "trigger": "AUTO",
 "magazineSize": 35,
 "reloadTime": 1.4,
 "multishot": 1
}
```

Melee record (Nikana Prime, trimmed):

```json
{"name": "Nikana Prime", "uniqueName": "/Lotus/Weapons/Tenno/Melee/Swords/PrimeKatana/PrimeNikana",
 "damagePerShot": [9.9000006, 9.9000006, 178.2, 0, ...], "totalDamage": 198,
 "criticalChance": 0.28, "criticalMultiplier": 2.4000001, "procChance": 0.27999997, "fireRate": 1.0833334,
 "masteryReq": 12, "productCategory": "Melee", "slot": 5, "omegaAttenuation": 1.2,
 "blockingAngle": 55, "comboDuration": 5, "followThrough": 0.69999999, "range": 2.5,
 "slamAttack": 594, "slamRadialDamage": 396, "slamRadius": 6, "slideAttack": 396,
 "heavyAttackDamage": 990, "heavySlamAttack": 792, "heavySlamRadialDamage": 594, "heavySlamRadius": 7, "windUp": 0.5}
```

Field notes:

| Field | Present on (of 841) | Meaning and notes |
|---|---|---|
| `uniqueName` | 841 | DE internal path, the **join key**. WFCD's `uniqueName` and the wiki's `InternalName` use the same string (see 1.8). |
| `name` | 841 | English display name. **Not unique**: 10 names repeat (for example Grimoire ×2, Mausolon ×3, and zaw tips with `PvPVariant…` twins). Archwing guns carry a `<ARCHWING> ` prefix token. |
| `damagePerShot` | 841 | Always 20 floats. Index order (WFCD `config/damageTypes.json`, and checked against single-element weapons here): 0 Impact, 1 Puncture, 2 Slash, 3 Heat (Ignis Wraith), 4 Cold (Glaxion), 5 Electricity, 6 Toxin (Torid), 7 Blast, 8 Radiation (Kuva Nukor), 9 Gas, 10 Magnetic, 11 Viral, 12 Corrosive (Catabolyst), 13 Void (Sirocco), 14 Tau (Haalvu), 15 Cinematic, 16 ShieldDrain, 17 HealthDrain, 18 EnergyDrain, 19 True (Shadow Clones). Slots 15-18 are 0 on every record. |
| `totalDamage` | 841 | Equals `sum(damagePerShot)` within 0.6 on every record. On parts it is 0 while the vector holds a 0.5 Magnetic placeholder (see below). |
| `criticalChance`, `criticalMultiplier`, `procChance` | 841 | Fractions. `procChance` is per projectile/pellet. Shotgun examples: Kuva Kohm 0.10 here vs wiki 0.9; Twin Kohmak 0.138 vs 0.69. |
| `fireRate` | 841 | Shots/s; melee attack speed. **Burst weapons give the in-burst rate**: Burston Prime 9.375 vs wiki 5.0; Kuva Hind 9.09 vs 5.0. Burst count and delay are missing. |
| `multishot` | 399 (ranged only) | Integer pellet count. Values: 1 (341), 6-15 for shotguns. |
| `magazineSize` | 385 | Missing on 14 ranged items that use no magazine: Grimoire, Nataruk, Convectrix, Helstrum, Artemis Bow, Regulators, Glory, Arquebex, and others. |
| `reloadTime` | 399 | Seconds. Present on all records that have `trigger`. |
| `trigger` | 399 | `AUTO` 158, `SEMI` 108, `CHARGE` 46, `HELD` 37, `BURST` 33, `ACTIVE` 10, `DUPLEX` 5, `Auto Burst` 2. **No charge time.** |
| `accuracy`, `noise` (`ALARMING`/`SILENT`) | 399 / 401 | Not needed for DPS. |
| `slot` | 661 | 0 secondary, 1 primary, 5 melee, 7 archgun, 14 railjack. Absent on all 180 modular parts. |
| `productCategory` | 841 | `Pistols` 329 (includes 180 modular parts), `Melee` 225, `LongGuns` 196, `SpecialItems` 36 (exalted weapons), `SentinelWeapons` 24, `SpaceGuns` 22, `SpaceMelee` 8, `OperatorAmps` 1 (Sirocco). Railjack: `CrewShipWeapons`. |
| `omegaAttenuation` | 841 | Riven disposition. 1.2 on 649 of the 659 damage-dealing slotted weapons, 0.5 on 9 (archwing and Ironbride), 1 on Sirocco. **This is correct, not a placeholder.** The wiki says dispositions were retired and normalized to 1.20x (`cache/wiki/Riven_Mods.wiki` lines 7, 784, 1011), and wiki `Module:Weapons/data` now has `Disposition = 1.2` with the old value kept in `_RetiredDisposition` (658 records across the weapon data files). This contradicts the "placeholder" note in `data-source-wfcd.md`. |
| `maxLevelCap` | 52 | 40 for Kuva/Tenet/Coda and similar weapons. Progenitor bonus elements are per instance and are not in the export. |
| Melee extras | 225-259 | `heavyAttackDamage` (224/225), `slamAttack`, `slamRadialDamage`, `slamRadius`, `heavySlam*`, `slideAttack`, `windUp`, `comboDuration`, `followThrough`, `range`, `blockingAngle`. These are **totals only**, with no per-type split. |
| `codexSecret`, `excludeFromCodex` (71), `description`, `masteryReq` | — | `excludeFromCodex` marks PvP variants and internal items. |

### Coverage (measured)

- 841 `ExportWeapons` + 143 `ExportRailjackWeapons` = 984 records. 659 `ExportWeapons` have `totalDamage > 0`.
- **182 records have `totalDamage == 0`.** All are parts or non-weapons, and each carries a dummy `damagePerShot[10] = 0.50006479`:
  - Zaw parts: ModularMelee01 (34), ModularMelee02 (5), ModularMeleeInfested (6).
  - Kitgun parts: SUModularSecondarySet1 (24), SUModularPrimarySet1 (4), InfKitGun (8).
  - Amp parts (24).
  - MOA, creature and Zanuka pet parts (49), K-Drive parts (20), and 8 other items.

  Example (Catchmoon chamber):
  ```json
  {"name": "Catchmoon", "uniqueName": "/Lotus/Weapons/SolarisUnited/Secondary/SUModularSecondarySet1/Barrel/SUModularSecondaryBarrelAPart",
   "damagePerShot": [0,0,0,0,0,0,0,0,0,0,0.50006479,0,0,0,0,0,0,0,0,0], "totalDamage": 0,
   "criticalChance": 0, "criticalMultiplier": 0, "procChance": 0, "fireRate": 0, "masteryReq": 0,
   "productCategory": "Pistols", "omegaAttenuation": 1.2}
  ```
  → **Kitgun and zaw stats must come from the wiki** (`Module:Weapons/data/modular`, `Module:Modular/data`).
- Against WFCD (`cache/wfcd/{Primary,Secondary,Melee,Arch-Gun,Arch-Melee,SentinelWeapons}.json`, 668 weapons): **all 668 WFCD uniqueNames are in the PE, and crit, status, fireRate, magazine, reload and totalDamage differ on 0 of them.** WFCD's base stats are exactly the PE values. The PE has 177 damage-dealing records that WFCD's weapon files lack: 143 railjack weapons, 32 exalted weapons (Exalted Blade, Artemis Bow, Shadow Claws, Desert Wind and others), and 2 enemy weapons.
- Top-level stats against wiki `Module:Weapons/data` attack 1 (309 weapons that a simple regex could pair by InternalName): crit chance agrees on 306, crit multiplier 306, status 300, fire rate 291. Every disagreement is a different attack or convention, not stale data:
  - Fulmin Prime and Epitaph Prime: the export describes the other fire mode.
  - Kuva Kohm and Twin Kohmak: the wiki gives status per shot, the export per pellet.
  - Burst weapons: the export gives in-burst fire rate.
  - Synoid Simulor: the wiki's attack 1 has CC 0.

### What ExportWeapons does not have

1. **Multiple attacks.** Only one damage vector per weapon. Sometimes two attacks are summed: Kuva Bramma's `damagePerShot` is Impact 187 + Blast 839, which is the wiki's "Charged Shot" plus its child "Radial Attack" (`AttackParentIndex = 1`). The cluster bombs (3 × 49 Impact, 3 × 57 Blast) are absent. Other times the AoE part is dropped: Lenz shows only 50 Impact + 10 Cold. **We cannot tell which case applies from the export alone.**
2. Charge time, burst count and delay, AoE radius and falloff, punch-through, projectile vs hitscan vs beam, forced procs, ammo cost, headshot multiplier overrides, explosion delay.
3. Incarnon form, Incarnon evolutions, alt-fires, and the exalted-weapon scaling formulas.
4. Kitgun and zaw part stats (above). Amp parts too.
5. Weapon compatibility tags (`PROJECTILE`, `AOE`, `SINGLESHOT`, `ASSAULT_AMMO` …). The wiki has them as `CompatibilityTags`. We need them for mod compatibility (bow vs rifle, for example).
6. Innate weapon passives. The wiki has them as `DefaultUpgrades` / `…InnateUpgrade` mods, for example `/Lotus/Weapons/Tenno/LongGuns/TnGlassShotgun/TnGlassShotgunInnateUpgrade`. They are not in ExportUpgrades either (see 1.3).

## 1.3 ExportUpgrades (mods)

Real record (Serration, levelStats trimmed to 3 of 11 ranks):

```json
{
 "uniqueName": "/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod",
 "name": "Serration",
 "polarity": "AP_ATTACK",
 "rarity": "UNCOMMON",
 "codexSecret": false,
 "baseDrain": 4,
 "fusionLimit": 10,
 "compatName": "Rifle",
 "type": "PRIMARY",
 "levelStats": [
  {"stats": ["+15% Damage"]},
  {"stats": ["+30% Damage"]},
  "...",
  {"stats": ["+165% Damage"]}
 ]
}
```

More real `levelStats` strings that show what parsing involves:

- Condition Overload r1: `"+13.3% Melee Damage per Status Type affecting the target"` (rounded; the real value is 80/6).
- Galvanized Chamber r10: `"+80% Multishot"`, `"On Kill:\r\n+30% Multishot for 20s. Stacks up to 5x."`
- Hunter Munitions r5: `"+30% chance to apply <DT_SLASH_COLOR> on Critical"` (the damage type is a colour token).
- Blood Rush r10: `"+40% Critical Chance stacks with Combo Multiplier"`.

Field notes:

| Field | Present on (of 1,603) | Notes |
|---|---|---|
| `uniqueName`, `name` | 1,603 | 134 names repeat. Serration has 3 records: normal (fusionLimit 10), `Beginner` (3), and `Intermediate` (5). Filter on `codexSecret` or path to drop the beginner and intermediate copies. |
| `polarity` | 1,603 | `AP_ATTACK` 529 (Madurai), `AP_TACTIC` 413 (Naramon), `AP_POWER` 239 (Zenurik), `AP_DEFENSE` 226 (Vazarin), `AP_PRECEPT` 94 (Penjaga), `AP_UNIVERSAL` 87, `AP_WARD` 9 (Unairu), `AP_UMBRA` 5, `AP_ANY` 1. |
| `rarity` | 1,603 | COMMON 230, UNCOMMON 412, RARE 880, LEGENDARY 81. |
| `baseDrain`, `fusionLimit` | 1,603 | Rank-0 drain and max rank. `len(levelStats) == fusionLimit + 1` on all 1,471 records that have `levelStats`. |
| `type` | 1,586 | Slot family: WARFRAME 418, PRIMARY 341, MELEE 203, SECONDARY 175, SENTINEL 94, STANCE 84, PARAZON 40, ARCH-GUN 39, AURA 36, ARCH-MELEE 20, ARCHWING 15, KAVAT 19, KUBROW 18, `---` 80. |
| `compatName` | 1,586 | **Coarse, human-oriented.** 240 distinct values. Rifle 119, Shotgun 119, Pistol 139, Melee 143, PRIMARY 10, Assault Rifle 15, Sniper 14, Bow 10, Thrown Melee 13, and so on. 63 values are warframe names (augments) and 89 are weapon names (weapon augments, for example `Vulkar`, `Lanka`, `Silva & Aegis`). **This is not a compatibility-tag system.** Vigilante Armaments is `PRIMARY`; Bladed Rounds is `Rifle`. Real rules (the bow vs rifle exclusions, AoE-only mods) need wiki `Module:Mods/itemCompatibility/data` and the weapon `CompatibilityTags`. |
| `levelStats[].stats[]` | 1,471 | **Display strings, not numbers.** 10,431 lines in total. 5,791 match a simple `^[+-]?N%? <words>$` pattern; 1,911 contain `<DT_…>` colour tokens. Values are rounded to 1 decimal. Conditional and stacking effects are free text. |
| `modSet` / `ExportModSet` | 72 / 19 | Set-bonus text per number of pieces, for example `"30% chance to enhance Critical Hits from Primary Weapons."` (Vigilante, `numUpgradesInSet: 6`). |
| `modSetValues` | 5 | Numeric set values exist only for the Umbral mods (Intensify `[0.25, 0.75]`, Fiber and Vitality `[0.3, 0.8]`) and Sacrificial Steel and Pressure (`[0.25, 0.5]`). |
| `subtype` | 220 | Warframe uniqueName for augments, for example `/Lotus/Powersuits/Volt/VoltBaseSuit`. |
| `isUtility` | 195 | Exilus-eligible flag. |
| `upgradeEntries` | **17** | Rivens and the Kuva/Tenet/railjack "Unfused Artifact" innate rolls. See 1.4. |
| (no `levelStats`) | 132 | Stances 78, Parazon 17, rivens and artifacts 17, plus a few others. **Stance combos are absent.** |

Also in this file: `ExportAvionics` (82, Railjack) and `ExportFocusUpgrades` (105, focus nodes as text). Neither is needed.

**Answer to the key question:** apart from rivens, ExportUpgrades has **no machine-usable per-rank numeric effects**. `levelStats` is localized display text. Parsing it would give rounded values (13.3% instead of 13.333%) and would need a hand-written grammar for conditional effects. The wiki's `Module:Mods/data` (separate finding) is the better source for effects. Use ExportUpgrades as the authoritative list of uniqueNames, polarity, drain, max rank and rarity, and as a regression check: our computed value at each rank, formatted to 1 decimal, should match the `levelStats` number.

## 1.4 Riven `upgradeEntries` (machine-usable)

Real excerpt (`/Lotus/Upgrades/Mods/Randomized/LotusRifleRandomModRare`, 3 of 38 entries):

```json
{"tag": "WeaponCritChanceMod", "prefixTag": "crita", "suffixTag": "cron",
 "upgradeValues": [{"value": 0.016666001, "locTag": "|val|% Critical Chance"}]}
{"tag": "WeaponFireRateMod", "prefixTag": "croni", "suffixTag": "dra",
 "upgradeValues": [{"value": 0.00667, "locTag": "|val|% Fire Rate (x2 for Bows)"}, {"value": 0.00667}]}
{"tag": "WeaponFactionDamageCorpus", "prefixTag": "manti", "suffixTag": "tron",
 "upgradeValues": [{"value": 0.0049999999, "locTag": "|val| Damage to Corpus"}]}
```

- 7 riven classes with entries: Rifle, Shotgun, Pistol, Melee, Archgun, Kitgun, Zaw. **Companion Weapon Riven has no `upgradeEntries` record** (only the `Raw…` veiled record).
- `value × 90 × disposition(1.2 since the retirement) × curse/buff multiplier` gives the stat. Check: Rifle CC 0.016666 × 90 = 150% and Damage 0.0183333 × 90 = 165%, both matching the wiki riven base table. The ×90 factor is my inference (UNVERIFIED in the code sense); cross-check it with `cache/wiki/Riven_Mods.wiki` line 784 and on.
- `tag` names the in-game stat (`WeaponCritChanceMod`, `WeaponStunChanceMod` = status chance, `WeaponFireIterationsMod` = multishot, `WeaponFactionDamageScaldra`/`Techrot` exist). That gives us a stable vocabulary for riven stat IDs. 63 distinct tags across all 17 records.
- Kuva/Tenet "Unfused Artifact" (`/Lotus/Weapons/Grineer/KuvaLich/Upgrades/InnateDamageRandomMod`) lists the 7 possible progenitor elements (`InnateHeatDamage` … `InnateMagDamage`) with `value: 0.1`. That is the minimum roll only; the 60% maximum is not in the export.

## 1.5 Arcanes (ExportRelicArcane)

Real record (Primary Merciless, levelStats trimmed):

```json
{"uniqueName": "/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamageOnKill", "name": "Primary Merciless",
 "codexSecret": true, "rarity": "RARE",
 "levelStats": [{"stats": ["On Kill:\r\n+5% Damage for 4s. Stacks up to 12x."]},
                {"stats": ["On Kill:\r\n+10% Damage for 4s. Stacks up to 12x."]}, "...",
                {"stats": ["On Kill:\r\n+30% Damage for 4s. Stacks up to 12x.\r\n+30% Reload Speed", "+30% Reload Speed"]}]}
```

- 172 arcanes (`/Lotus/Upgrades/CosmeticEnhancers/{Offensive 76, Utility 30, Defensive 25, OperatorArmour 19, Zariman 12, OperatorAmps 10}`). 168 have `levelStats` (142 with 6 ranks, 26 with 4). The 4 without are `Arcane Steadfast` copies.
- **No slot or compatibility field and no numeric values**, only text as for mods. The final rank's text includes the "+1 Arcane Revive" line.
- The PE and WFCD have the same 172 uniqueNames. Wiki `Module:Arcane/data` lists 5 `/Lotus/Upgrades/CosmeticEnhancers/Antiques/*` arcanes (for example "Zid-An Haras", "Zid-An Osbok") that **neither PE nor WFCD has**. Either they are newer or they are not exported.
- The other 3,201 records are relics (`relicRewards`), which we do not need.

## 1.6 Archon shards, Incarnon, warframes, companions, enemies

- **Archon shards:** 12 records in ExportResources (6 colours × normal/Tauforged). The name carries a token, for example `"<SHARD_RED_SIMPLE> Crimson Archon Shard"`, `uniqueName` is `/Lotus/Types/Gameplay/NarmerSorties/ArchonCrystalAmar[Mythic]`, and `parentName` is `.../ArchonCrystal[Mythic]`. **There are no effect options or values.** The shard bonus table must come from the wiki or be hand-written.
- **Incarnon:** only the 45 `… Incarnon Genesis` adapter items (`/Lotus/Types/Items/MiscItems/IncarnonAdapters/...`), each with a one-line description. Example: Torid: "Awaken this weapon's ability to fire a long-range <DT_POISON_COLOR>Toxin beam in Incarnon Form." There are **0 `/Lotus/Upgrades/Evolutions/*` records** in any manifest. The wiki's Mods data references 10 such evolution paths.
- **Warframes:** 127 records (`Suits` 119, `SpaceSuits` 5, `MechSuits` 2). Fields: `health`, `shield`, `armor`, `power`, `sprintSpeed`, `abilities[]` (name + description text), `passiveDescription` with unresolved placeholders (`|SPEED|`, `|RELOAD|`), and `exalted[]` (exalted weapon uniqueNames). Excerpt: `{"name":"Mesa","health":365,"shield":180,"armor":105,"power":100,"exalted":["/Lotus/Powersuits/Cowgirl/SlingerPistols"]}`. There are no ability damage numbers.
- **Companions:** ExportSentinels has 34 records (health/shield/armor/power), last modified 2025-03-18. Companion weapons are in ExportWeapons (`SentinelWeapons` 24).
- **Enemies: not exported.** The index has no ExportEnemies, and `.../MobileExport/Manifest/ExportEnemies.json` is 404. The only enemy-named records are Specter consumables in ExportGear, for example `"Corrupted Heavy Gunner Specter"` (`/Lotus/Types/Restoratives/Consumable/CorruptedHeavyGunnerBall`). Faction, base level, health, shield, armor, overguard and damage reduction must come from wiki `Module:Enemies/data/*`.

## 1.7 Freshness

- The live world state (`https://api.warframestat.us/pc`, fetched 2026-11-23T21:17Z) has `buildLabel` `2026.10.28.09.54/8e7dGXr3Omgob8q-e55qeg`. Its newest patch news item is "Hotfix 44.1.4" (2026-10-27). The PE index `Last-Modified` is 2026-10-28 17:14 GMT, the same day as the build label. Weapons, Upgrades and RelicArcane manifests are from 2026-10-22, so 44.1.4 did not change them. **Conclusion: on 2026-11-23 the Public Export on content.warframe.com matches the live PC build.**
- DE's own world-state host `api.warframe.com` was blocked by this sandbox's proxy, so `buildLabel` was read through warframestat.us.
- Staleness risk: WFCD warns that content.warframe.com can lag origin.warframe.com. Untested here (see Open questions).
- Fast update detection for a build tool: GET the index with `If-None-Match`. If any line's hash changed, refetch only that manifest.

## 1.8 Join keys to WFCD and the wiki

| Pair | Measured match on uniqueName |
|---|---|
| WFCD weapons → PE | 668 / 668 |
| Wiki `Module:Weapons/data/*` `InternalName` → PE weapons | 825 / 872. The rest are mostly beast companion weapons (`/Lotus/Types/Friendly/Pets/BeastWeapons/*`), `/Lotus/Powersuits/Fairy/FlightAgentGun`, and `/Lotus/Powersuits/Temple/ExaltedGuitar`. |
| Wiki `Module:Mods/data` `InternalName` → PE upgrades | 1,451 / 1,635. Of the unmatched entries I could pair with names, 149 are not in PE: weapon innate/passive upgrades (`…InnateUpgrade`, `…InnateMod`), 10 Incarnon evolutions (`/Lotus/Upgrades/Evolutions/...`), DirectorMods, and syndicate weapon procs. |
| Wiki `Module:Arcane/data` → PE arcanes | 161 / 166 (the 5 Antiques). |

`uniqueName` is a stable key across all three sources, so the pipeline should join on it, never on `name`.

## 1.9 Rust build-tool notes

- Decompress the index with `lzma-rs` (`lzma_rs::lzma_decompress`, which reads the legacy `.lzma` header) or with `xz2` using `Stream::new_lzma_decoder`. The `xz` format decoder will reject it.
- Split lines on `\r?\n`, skip empty lines, and **trim `\r`**. Build the URL as `format!("https://content.warframe.com/PublicExport/Manifest/{line}")` with no percent-encoding. (`reqwest` leaves `+` and `!` unchanged in the path; check this in a unit test.)
- Parse with `serde_json`, using `#[serde(default)]` for the many optional fields. Keep `damagePerShot` as `[f32; 20]`, map it by index, and round to 6 significant digits.
- Cache: `cache/public-export/<file>!<hash>`. The index is the only mutable input. About 1 request/s is ample; a full fetch is 17 requests (~15 MB).
- Filter: `totalDamage > 0`, `slot.is_some()`, and drop `excludeFromCodex` PvP variants (71) unless the wiki references them.

---

# Part 2: warframe-connector (abandoned project) — what we can reuse

Read: `README.md`, `DESIGN.md`, `src/wfconnector/{gamedata,catalog,polite}.py` (plus a grep of the other modules for URLs).

**Data sources it used** (none of them give damage data):

| Source | URL | Relevance to us |
|---|---|---|
| warframe.market v2/v1 | `https://api.warframe.market/v2`, `/v1/items/{slug}/statistics` | None (prices). |
| warframestat.us world state | `https://api.warframestat.us/pc` | Only for `buildLabel` (game-version check, see 1.7). |
| warframestat.us WFCD mods | `https://api.warframestat.us/mods?filter=tradable:true&only=name,tradable,drops,isAugment,compatName` | Shows that warframestat.us serves WFCD items with field filtering. We should pin WFCD files directly instead. |
| DE weekly riven prices | `https://www-static.warframe.com/repos/weeklyRivensPC.json` (a **JS object literal, not JSON**) | Prices only; no disposition or stat data. |
| DE usage stats | `https://www-static.warframe.com/repos/WarframeUsageData{year}.json` | None. |

It **never touched the Public Export**, the wiki modules, or enemy data.

**Patterns worth copying into the Rust data-fetch tool:**

1. `PoliteClient` (`polite.py`): a per-host start-to-start interval (default 2 req/s); one lock per host so retries hold the slot; and single-flight de-duplication of identical in-flight requests. It retries only on 429/503/509 and connection errors, up to 3 attempts, using `Retry-After` (capped at 60 s) or 1/2/4 s back-off. It caches 404s for the TTL (so missing resources are not re-probed) and caches nothing else that failed. It sends a descriptive `User-Agent` and keeps the contact optional (`WFC_CONTACT`). A Rust equivalent is `reqwest` + `tokio::sync::Mutex` per host. A build-time fetcher can be simpler (sequential, about 1 req/s) but should keep the same retry and 404 rules.
2. Name resolution (`catalog.py`): normalize with casefold and collapsed whitespace. The order is: exact name → exact slug → unique substring hit → `Ambiguous` (≤8 candidates, shortest name first) → `NotFound` with suggestions ranked by shared words. This suits our frontend item search and any CLI that takes weapon names. In our case the resolver must return `uniqueName`s, because names repeat (1.2, 1.3).
3. `js_literal_to_json` (`gamedata.py`) converts DE's JS-literal files. We only need it if we ever read `www-static.warframe.com/repos/*`.
4. Architecture rule from `DESIGN.md`: "wire shapes never leave the boundary modules". Tests use trimmed fixtures captured from live APIs (`tests/fixtures/`) and never touch the network. We should do the same: one module per upstream (public_export, wfcd, wiki) that parses into our own types, with golden tests on cached fixtures.

**Lessons recorded in DESIGN.md that carry over:** observe live endpoints before trusting docs. The project found that v1 orders return 403 and that warframestat.us uses mission type `"Extermination"`. In this evaluation the parallel finding is that the documented MobileExport URLs are 404 and that the manifest URL must keep `+` unencoded.

---

## Open questions

1. **Licence and terms of the Public Export.** No licence file or header is served. WFCD redistributes it under MIT, but DE's terms for redistributing derived data in a local app are not stated. The manager or user should decide (bundling derived stats is common practice in community tools).
2. Is `origin.warframe.com` ever ahead of `content.warframe.com`, and by how long after a hotfix? It was blocked here, so this is untested.
3. Which weapons have a `damagePerShot` that sums several attacks (Kuva Bramma) and which drop the AoE (Lenz)? We need a systematic diff against wiki attacks to decide how the pipeline reconciles them. The proposed rule is that wiki attacks are authoritative and PE top-level stats are a sanity check.
4. The riven ×90 scaling factor and the exact buff/curse multipliers by stat count: inferred from two values here. Confirm against the wiki riven formula before coding.
5. Are the 5 "Antiques" arcanes (Zid-An …) live in-game and deliberately unexported, or new? This affects whether the wiki is the only source for some current items.
6. `procChance` per pellet vs the wiki's per-shot status for multi-pellet weapons (Kuva Kohm 0.10 vs 0.9): confirm which convention the wiki module uses for each weapon before using either in the engine.
