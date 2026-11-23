Status: draft

# Data source: wiki.warframe.com Lua data modules

Evaluated 2026-11-23. Everything below was measured on modules downloaded that day. Raw copies are in
`research/external/cache/wiki-modules/raw/` (one `.lua` per module, `/` replaced by `_`). JSON conversions are in
`research/external/cache/poc/out/`. Analysis scripts are in this session's scratchpad and were not kept. Every count
below can be reproduced with a few lines of Python over the JSON files.

## TL;DR

- **Weapons are the strongest part of this source.** `Module:Weapons/data/<slot>` has 911 weapon entries (767 not
  marked `_IgnoreEntry`) and 1984 attack records. Each attack has a per-type damage map, crit chance and multiplier,
  status chance, fire rate, multishot, falloff, AoE range, forced procs, charge time, burst count and delay, spool,
  and punch-through. Incarnon *forms* are included as extra attacks. The data is pure Lua table literals, edited
  actively (last edits Oct–Nov 2026), and `Disposition` already reflects the Hotfix 44.1.3 riven-disposition
  retirement.
- **Enemies are good:** 1019 entries across 12 faction sub-modules, with faction, `BaseLevel`, `Health`, `Shield`,
  `Armor`, `Overguard`, `EximusHealth`/`EximusShield`, body-part multipliers (as prose strings) and `ProcResists`.
  There is **no** damage-reduction field (Eidolons, Archons, bosses).
- **Damage-type and faction modifiers are good.** `Module:DamageTypes/data` has the post-U36 faction tables, for
  example Grineer +50% Corrosive and Impact, Kuva Grineer −50% Heat.
- **Mods and arcanes are weak.** They carry only *max-rank prose* (`Description`) plus `UpgradeTypes` enum tags. There
  are no per-rank numbers and no machine-usable conditions. Per-rank tables exist only as hand-written wikitext on
  each mod's page.
- **Missing entirely:** Incarnon *evolution perks* (they are wikitext tables on `<Weapon> Incarnon Genesis` pages),
  archon shards, Kuva/Tenet bonus-element values, beam chaining, and status-effect DoT and duration numbers.
- **Parsing:** the vendored Lua 5.1 in the `mlua` crate builds here in about 10 s. It converted all 37 modules
  tried, in about 50 ms per module. A no-execution table-literal parser gives byte-identical JSON for every pure-data
  module. It does not work for `DamageTypes/data` and `Stances/data`, which run code at load. Recommendation: use
  `mlua`.
- **License:** the wiki says **CC BY-NC-SA 3.0**, not CC BY-SA (from `meta=siteinfo&siprop=rightsinfo`).

## 1. Access, license, etiquette

| Item | Finding |
|---|---|
| Raw module source | `https://wiki.warframe.com/w/Module:<Name>?action=raw`. Returns `text/x-wiki`, `Last-Modified` = last edit time, served behind Cloudflare (`cf-cache-status`), `cache-control: private, must-revalidate, max-age=0`. |
| Batched fetch | `api.php?action=query&prop=revisions&rvprop=content\|timestamp\|ids&rvslots=main&titles=A\|B\|...&format=json&formatversion=2&maxlag=5`. Tested with 2 titles: the content is byte-identical to `?action=raw`, and `contentmodel` is `Scribunto`. MediaWiki accepts up to 50 titles per query, so every needed module fits in one or two requests (about 6 MB total). Not tested: whether a 50-title batch hits the API result-size limit. |
| Module discovery | `api.php?action=query&list=allpages&apnamespace=828&aplimit=500` returned 662 Module pages in 2 requests (continue token `apcontinue`). List cached in `cache/wiki-modules/allpages.txt`. |
| Freshness check | `prop=revisions&rvprop=timestamp\|user\|comment` for the modules (cached in `revs.json`). The build tool can store `revid` per module and skip unchanged ones. |
| License | `rightsinfo`: **CC BY-NC-SA 3.0** (`https://creativecommons.org/licenses/by-nc-sa/3.0/`). A derived bundled data file must carry attribution, stay non-commercial, and be share-alike. This fits a free local tool, but the shipped data file needs a LICENSE/NOTICE. |
| Rate limits | No rate-limit headers were seen (`x-ratelimit*` or `retry-after` absent). The site runs MediaWiki 1.45.3 behind Cloudflare. Etiquette: identify with a descriptive User-Agent, send `maxlag=5` on API calls, make requests one at a time (about 1 req/s, as used here), and cache by `revid`. A full refresh is under 60 requests with `?action=raw`, or about 2 batched API calls. |

## 2. Module inventory (useful ones)

Sizes are of the raw Lua. "Code?" means the module executes logic at load, beyond a table literal.

| Module | Size | Last edit (UTC) | Code? | Use |
|---|---|---|---|---|
| `Weapons/data` | 4.7 KB | — | yes (index; `mw.loadData` of sub-pages) | Not needed. Fetch the 8 sub-pages directly. |
| `Weapons/data/primary` | 341 KB | 2026-11-14 | no | 200 weapons |
| `Weapons/data/secondary` | 235 KB | 2026-10-22 | no | 156 |
| `Weapons/data/melee` | 473 KB | 2026-10-22 | no | 250 |
| `Weapons/data/archwing` | 81 KB | 2026-11-02 | no | 50 |
| `Weapons/data/companion` | 41 KB | 2026-10-22 | no | 41 |
| `Weapons/data/modular` | 44 KB | 2026-10-24 | no | 39 (16 amps, kitgun/zaw base entries) |
| `Weapons/data/railjack` | 167 KB | 2026-10-09 | no | 157 (144 are `_IgnoreEntry` Mk I/II/III variants) |
| `Weapons/data/misc` | 13 KB | 2026-08-09 | no | 18 (emplacements, parazon, fishing spears, unique) |
| `Weapons/ppdata` | 1.7 MB | — | no | Preprocessed percentile and top-3 stats per class. **Not useful.** |
| `Mods/data` | 942 KB | 2026-11-01 | `local modData = {...} return modData` | 1550 mods, 19 sets, 89 innate "DefaultUpgrades" |
| `Arcane/data` | 96 KB | 2026-11-21 | no | 166 arcanes |
| `Enemies/data` | 4.9 KB | — | yes (index) | Lists the 12 live faction sub-pages (`COOP_FACTIONS`) |
| `Enemies/data/{grineer,corpus,infestation,orokin,sentient,stalker,narmer,themurmur,techrot,scaldra,anarchs,unaffiliated}` | 6.7–308 KB | 2026-04 … 2026-09 | no | 1019 enemies |
| `Enemies/data/pre-U36` | 479 KB | 2024-06-27 | no | Archive of the old health classes (849 entries). **Stale by design. Do not use.** |
| `DamageTypes/data` | 48 KB | 2026-11-13 | **yes** (builds `total` table, colours) | Faction/health modifiers, damage types, procs |
| `Modular/data` | 32 KB | 2026-06-22 | no | Kitgun chamber/grip/loader, Zaw strike/grip/link, MOA/Hound parts |
| `Modular/data/kitgun` | 1.28 MB | 2026-06-20 | no | 1200 pre-expanded kitgun combos (chamber × loader × grip) |
| `Stances/data` | 223 KB | 2026-10-02 | **yes** (`addSharedCombos`) | 132 stances: per-hit damage %, hit counts, procs |
| `Warframes/data` | 169 KB | 2026-11-10 | no | 126 warframes, 5 archwings, 2 necramechs, 2 operators |
| `Ability/data`, `Ability/data/stats` | 203 / 180 KB | 2026-11-13 | partly | 339 abilities (prose) and 220 stat blocks (`Label`/`Modifier`/`Values`) |
| `Focus/data` | 109 KB | 2026-01-09 | no | 5 schools, 104 ways (rank stats as prose) |
| `Companions/data` | 72 KB | 2026-02-15 | no | 41 companions |
| `Upgrades/data` | 11 KB | — | no | Enum: `UpgradeTypes` name → int id |
| `Enum/data` | 9 KB | — | no | Game enums (polarity ids, etc.) |
| `Mods/itemCompatibility/data` | 8 KB | — | no | Mod "Type" → internal compatibility path |
| `Flaicher/dispoData` | 1.6 KB | 2020-11-03 | no | Old user sandbox of melee dispositions. **Stale.** Ignore. |
| `Maximization/data` | 235 KB | — | yes (functions) | Ability-calculator UI config (expressions as strings). Not useful. |

Modules searched for and **not found** (no Module page exists): Incarnon / evolutions, Archon shards, Rivens /
riven stats, status-effect math. `Tooltips/data/*` and `Stats/data` only re-export icon tables.

## 3. Weapons (`Module:Weapons/data/<slot>`)

### Record (Acceltra, trimmed; real excerpt)

```lua
Acceltra = {
    Accuracy = 23.5, AmmoMax = 96, AmmoPickup = 40, AmmoType = "Primary",
    Attacks = {
        {   AmmoCost = 1, AttackIndex = 1, AttackName = "Rocket Impact",
            CritChance = 0.32, CritMultiplier = 2.8,
            Damage = { Impact = 35 },
            FireRate = 12, ForcedProcs = { "Impact" }, IsSilent = false,
            MaxSpread = 7, MinSpread = 1.5, Multishot = 1, PunchThrough = 0,
            ShotSpeed = 70, ShotType = "Projectile", StatusChance = 0.06 },
        {   AttackIndex = 2, AttackName = "Rocket Explosion", AttackParentIndex = 1,
            CritChance = 0.32, CritMultiplier = 2.8,
            Damage = { Puncture = 35.2, Slash = 8.8 },
            Falloff = { EndRange = 4, Reduction = 0.5, StartRange = 0 },
            FireRate = 12, IsSilent = false, Multishot = 1, PunchThrough = 0,
            Range = 4, ShotType = "AoE", StatusChance = 0.06 }
    },
    Class = "Rifle", CompatibilityTags = { "PROJECTILE", "AOE", "ASSAULT_AMMO" },
    DefaultUpgrades = { "/Lotus/Weapons/Tenno/LongGuns/SapientPrimary/SapientPrimaryInnateUpgrade", ... },
    Disposition = 1.2, _RetiredDisposition = 0.65, ExilusPolarity = "Madurai", Family = "Acceltra",
    InternalName = "/Lotus/Weapons/Tenno/LongGuns/SapientPrimary/SapientPrimaryWeapon",
    Introduced = "25.7", Magazine = 48, Mastery = 8, MaxRank = 30, Name = "Acceltra",
    Polarities = { "Naramon" }, Reload = 2, Slot = "Primary", Traits = { "Tenno" }, Trigger = "Auto",
    _TooltipAttackDisplay = 2
},
```

### Weapon-level fields (count out of 911 entries that have the field)

| Field | Count | Notes |
|---|---|---|
| `Name`, `Slot`, `Class`, `Attacks`, `Link`, `Image`, `Introduced` | 911 | `Class` values: Rifle, Shotgun, Pistol, Bow, Sniper Rifle, Launcher, Nikana, Zaw "Scythe / Heavy Blade", Turret, etc. |
| `InternalName` | 902 | `/Lotus/...` unique name. **This is the join key to DE Public Export / WFCD.** |
| `Trigger` | 616 | Free text: `Auto`, `Semi-Auto`, `Held`, `Charge`, `Auto-Spool`, `Burst`, `Duplex`, and compound forms such as `"Burst / Semi"`, `"Auto / Mag Burst"` (33 distinct strings). Attacks may override it with their own `Trigger` (265 attacks). |
| `Magazine`, `Reload`, `AmmoMax` | 612 / 609 / 609 | All 356 non-ignored primaries and secondaries have them. `AmmoMax = math.huge` on companion, railjack and some misc weapons. |
| `ReloadStyle`, `ReloadDelay`, `ReloadRate`, `ReloadDelayEmpty` | 202 / 199 / 196 / 3 | Battery / regenerating magazines. |
| `Spool` | 38 | Shots to reach full spool (Gorgon `Spool = 9`). |
| `SniperComboMin`, `SniperComboReset` | 15 / 17 | Sniper combo counter. |
| `MaxRank` | 737 | 30 or 40. All 48 lich weapons with a rank have 40. |
| `Disposition` | 660 | **Always 1.2.** The wiki changed every weapon to 1.2 in edits tagged "H44.1.3 riven disposition retirement". `Riven_Mods` page: "All weapons no longer have varying Dispositions and have been normalized to … (1.20x)". The old values are kept in `_RetiredDisposition` (658). |
| `CompatibilityTags` | 605 | Internal mod-compatibility tags (`PROJECTILE`, `AOE`, `BEAM`, `SNIPER_AMMO`, `SINGLESHOT`, stance tags). Missing on 72 of 356 guns (Hek, Boar, Kohm, ...). |
| `DefaultUpgrades` | 237 | Innate upgrade paths, resolved in `Mods/data.DefaultUpgrades` (89 entries). Lich weapons point to `/Lotus/Weapons/Grineer/KuvaLich/Upgrades/InnateDamageRandomMod`, which has **no value or element**. Incarnon weapons list `/Lotus/Upgrades/Evolutions/...` ids (10 such entries) with empty `Description`. |
| `IsLichWeapon` | 52 | Flag only. The bonus element and its 25–60% range are not here. |
| `IncarnonImage`, `IncarnonChargeGain`, `IncarnonDuration` | 89 / 68 / 24 | Marks an Incarnon-capable weapon (93 weapons total have an `Incarnon` attack or an `IncarnonImage`). `IncarnonDuration` is for melee (180 s). |
| Melee: `HeavyAttack`, `SlideAttack`, `SlamAttack`, `HeavySlamAttack`, `SlamRadius`, `HeavySlamRadius`, `WindUp`, `FollowThrough`, `MeleeRange`, `SweepRadius`, `BlockAngle`, `ComboDur`, `StancePolarity` | ~250 each (slam fields 19–21) | `HeavyAttack` is a **single total number** (Ack & Brunt `745`) with no type split. The type split is assumed to follow the Normal attack. **UNVERIFIED**, see Open questions. |
| `Traits` | 700 | `Prime`, `Kuva Lich`, `Tenet`, `Incarnon`, `Zaw`, `Kitgun`, `Vaulted`, ... |
| `_IgnoreEntry` | 145 | 144 are `true` (all railjack Mk I/II/III variants); Cinta has `_IgnoreEntry = false`. Skip the `true` ones. |
| `Polarities`, `ExilusPolarity`, `Mastery` | 538 / 356 / 692 | `Polarities` is missing on 74 guns (Braton, Boar, ...), probably meaning "no polarities", but undocumented. |

### Attack-level fields (count out of 1984 attacks)

| Field | Count | Notes |
|---|---|---|
| `AttackName`, `Damage`, `CritChance`, `CritMultiplier`, `StatusChance`, `FireRate` | 1984 | `CritChance`/`StatusChance` are fractions (0.32 = 32%). `Damage` is a **per-pellet** map, `{Type = value}`. Hek: `{Impact=11.25, Puncture=48.75, Slash=15}` × `Multishot = 7` = 525. `StatusChance` is also per pellet (Hek 0.107). Damage keys seen: Impact, Puncture, Slash, Blast, Heat, Electricity, Toxin, Radiation, Cold, Viral, Corrosive, Magnetic, Void, Gas, Tau, Finisher, True. `FireRate` is attacks/s (melee: attack speed). |
| `AttackIndex`, `AttackParentIndex` | 1965 / 239 | `AttackParentIndex` links an explosion to its impact (Acceltra 2 → 1). This is how to model "direct hit + AoE" as one shot. It is used inconsistently: Ack & Brunt's slams have `AttackParentIndex = 2` while Nikana Prime's slams have none. |
| `ShotType` | 1652 | `Hit-Scan` 360, `Projectile` 499, `AoE` 760 (480 of these are melee slam/heavy-slam), `Thrown` 26, `DoT` 5, plus a typo `Hitscan` ×2. Missing on 12 gun attacks (e.g. Hind "Semi-Auto Mode"). |
| `Multishot` | 1098 | Absent on melee attacks. Also absent on 11 gun attacks (e.g. Cinta's attacks, Alternox "Alt-Fire Explosion"), so default it to 1. |
| `Falloff = {StartRange, EndRange, Reduction}` | 1043 | `Reduction` = max damage removed at `EndRange` (Hek 0.8 → 20% damage left at 20 m). For `AoE` attacks this doubles as the **radial falloff**: `StartRange = 0`, `EndRange` = radius (Acceltra 4 m, Reduction 0.5). |
| `Range` | 1183 | Hitscan max range, or beam length (Ignis Wraith 27), or AoE radius. `math.huge` on Tunguska Cannon. |
| `ForcedProcs` | 716 | List of proc names: `"Impact"`, `"Heat"`, `"Cold"`, `"Lifted"`, `"Knockdown"`... Also a typo `ForcedProc` ×2 (Shraksun Scaffold). |
| `ChargeTime` | 158 | Seconds (Lanka 1.0, Lenz 1.2). Lanka's charged attacks have `FireRate = math.huge`, so effective rate must come from `ChargeTime`. |
| `BurstCount`, `BurstDelay`, `BurstReloadDelay` | 62 / 68 / 7 | Hind: `BurstCount = 5, BurstDelay = 0.12, FireRate = 5`. |
| `PunchThrough` | 1047 | Metres. `math.huge` on Fluctus. |
| `ShotSpeed` | 516 | m/s. |
| `IncarnonCharges` | 67 | Incarnon form "magazine" (Furis 280). |
| `ExplosionDelay`, `EmbedDelay`, `EffectDuration` | 24 / 3 / 20 | Delayed detonations (Lenz bubble 1.3 s, Gorgon Incarnon 1.1 s). |
| `ExtraHeadshotDmg` | 14 | Additive headshot bonus (Cernos Prime 0.5, Alternox −2). Semantics not documented. |
| `AmmoCost` | 783 | Beams 0.5. Typo `Ammocost` ×1. |
| `Accuracy`, `MinSpread`, `MaxSpread`, `IsSilent`, `Trigger`, `Reload` (per-attack) | various | `IsSilent` typo `IsSlilent` ×1. |

Attacks per weapon: 1 → 401 weapons, 2 → 148, 3 → 272 (mostly melee: normal + slam + heavy slam), 4 → 47,
5 → 25, 6+ → 18 (up to 12). The `_TooltipAttackDisplay` index (905) says which attack the wiki shows by default.
That is a reasonable default "primary attack" for the UI.

Not present at all: **AoE radius as a separate field** (use `Range` / `Falloff.EndRange`), **beam chain / extra
targets** (Kuva Nukor has no chain count. The inspiration's description says "hit up to two additional
targets"), **status duration / DoT tick values**, **headshot multiplier per weapon** (only the 14
`ExtraHeadshotDmg`), **zoom levels with stat changes** (`Zoom` only on 16 weapons), and **damage per heavy-attack
type**.

### Incarnon

- **Forms: present.** They are extra attacks named `"Incarnon Form"` (63), `"Incarnon Form AoE"` (14), `"Incarnon
  Form Radial Attack"` (9), `"Incarnon Form Charged Shot"` (4), and so on. Each has its own crit, status, fire rate,
  `Trigger`, `IncarnonCharges`, and falloff. Furis's form matches the inspiration sample exactly (crit 0.26 / 3.4×,
  status 0.24, 12/s, Heat 100, 280 charges).
- **Evolutions (perk choices per tier): absent from all modules.** They live only in wikitext tables on
  `<Weapon> Incarnon Genesis` pages (cached `cache/wiki-modules/Furis_Incarnon_Genesis.wiki`). They are written as
  prose with `X = 28 / X = 34` per-variant values. Example: "Haven Foray: Increase Base Damage by +X. With Overshields:
  Increase Base Damage by +30". The weapon's `DefaultUpgrades` `/Lotus/Upgrades/Evolutions/...` ids resolve to
  `Mods/data.DefaultUpgrades` entries with empty descriptions. The inspiration hand-encodes these perks as
  `action` objects. We need another source (DE export / WFCD) or hand-authored data.

### Kitgun / Zaw (`Module:Modular/data`, `Module:Modular/data/kitgun`)

```lua
KitgunSecondary = { Chamber = { Catchmoon = {
    CritChance = 0.21, CritMultiplier = 2, StatusChance = 0.21, ForcedProcs = { "Impact" },
    Damage = { Base = { Heat = 167, Impact = 89 }, Gibber = { Heat = 119, Impact = 41 },
               Haymaker = { Heat = 269, Impact = 191 }, ... },          -- damage depends on GRIP
    FireRate = { Gibber = 3.17, Haymaker = 1.17, Lovetap = 1.5, ... },  -- fire rate depends on GRIP
    Falloff = { StartRange = 8, EndRange = 16, Reduction = 0.9375 },
    Magazine = { Lowest = 5, Low = 7, Med = 9, High = 11, Highest = 13, SuperHighest = 17, ... },  -- tier picked by LOADER
    ... } } ,
  Grip = { Haymaker = { Name = "Haymaker", Recoil = 1 }, ... },
  Loader = { Splat = { CritChance = 0.14, CritMultiplier = 0.3, Magazine = "High", Reload = 1.7, StatusChance = -0.08 }, ... } }
Zaw = { Strike = { ["Plague Keewar"] = { CritChance = 0.18, CritMultiplier = 2, StatusChance = 0.22,
          Damage = { Impact = 88, Puncture = 57, Slash = 91, Viral = 70 }, Speed = -0.033,
          Type = { OneHanded = "Scythe", TwoHanded = "Staff" },
          Multi = { OneHanded = { Base = 1, Slide = 2, ... }, TwoHanded = { Base = 0.85, ... } } } },
        Grip = { Peye = { Damage = -4, Speed = 1, Type = "OneHanded" } }, Link = { ... } }
```

- Kitgun parts: 6 chambers, 5 grips per slot (secondary and primary grips are different), and 20 loaders (some are
  `_`-variants). Loaders add to crit, crit multiplier and status, and pick a magazine tier. Grips set damage and fire
  rate *per chamber*, as hard-coded tables (the module header says grips are "a vector rather than a scalar").
- `Modular/data/kitgun` pre-expands all **1200** combos (`"Catchmoon-Arcroid-Brash"` = chamber-loader-grip) into
  full `Weapons/data`-schema records. Cross-check: for all 1200 combos, attack 1's `Damage` and `FireRate` equal
  the chamber's `Damage[grip]` / `FireRate[grip]` in `Modular/data`, with **0 mismatches**. The combos were
  generated 2026-06-20, two days *before* the last parts edit ("Corrected Kitgun stats" 2026-06-22). Damage and
  fire rate still agree, but other fields could have drifted. It is safer to regenerate from parts.
- Zaw: 11 strikes, 10 grips, 16 link entries. **Not pre-expanded.** Final damage = strike damage + grip offset
  (`Damage = -4`), and speed similarly. Link effects are not fully parsed in this evaluation.
- Amps: 16 entries in `Weapons/data/modular` (as whole amps, not prism/scaffold/brace parts).

## 4. Mods (`Module:Mods/data`)

```lua
Serration = {
    BaseDrain = 4, CodexSecret = false,
    Description = "+165% Damage",
    Incompatible = { "Amalgam Serration", "Higasa Serration", "Flawed Serration", "Spectral Serration" },
    InternalName = "/Lotus/Upgrades/Mods/Rifle/WeaponDamageAmountMod",
    MaxRank = 10, Name = "Serration", Polarity = "Madurai", Rarity = "Uncommon",
    Tradable = true, Transmutable = true, Type = "Rifle",
    UpgradeTypes = { "WEAPON_DAMAGE_AMOUNT" }
},
```

Top-level tables: `Mods` (1550), `Sets` (19, set bonus as prose, `NumUpgradesInSet`), `DefaultUpgrades` (89 innate
weapon upgrades).

| Field | Count / 1550 | Notes |
|---|---|---|
| `Description` | 1550 | **Max-rank effect as prose**, with `\r\n` line breaks and `<DT_*_COLOR>` damage-type tokens (15 distinct). Examples: `"+80% Melee Damage per Status Type affecting the target"` (Condition Overload), `"+80% Multishot\r\nOn Kill:\r\n+30% Multishot for 20s. Stacks up to 5x."` (Galvanized Chamber). |
| `MaxRank`, `BaseDrain`, `Polarity`, `Rarity`, `Type` | 1545–1550 | `Type` is the compatibility class (`Rifle`, `Shotgun`, `Pistol`, `Melee`, `Primary`, `Sniper`, `Assault Rifle`, stance class names, or a **specific weapon/warframe name** for augments and exclusives: 262 distinct values). |
| `UpgradeTypes` | 856 | Internal upgrade enum per *effect line* (177 distinct). Examples: `WEAPON_DAMAGE_AMOUNT`, `WEAPON_FIRE_ITERATIONS` (multishot), `WEAPON_PERCENT_BASE_DAMAGE_ADDED` (elemental, 147), `GAMEPLAY_FACTION_DAMAGE`, `WEAPON_DAMAGE_IF_VICTIM_PROC_ACTIVE` (CO-type, 4 mods). This tells *what stat*, but not the *value*, condition or stacking. Ids are in `Upgrades/data`. |
| `Incompatible`, `IncompatibilityTags`, `CompatibilityTags` | 475 / 152 / 103 | Mutual exclusion (Serration vs Amalgam Serration) and tags like `POWER_WEAPON`, `SENTINEL_WEAPON`. |
| `IsExilus`, `IsFlawed`, `IsAbilityAugment`, `IsWeaponAugment`, `Class` (Galvanized, Amalgam, Riven, ...), `Set` | 192 / 141 / 226 / 74 / 80 / 72 | |

Machine-readability (weapon-type mods, 686 non-ignored): 525 have `UpgradeTypes`. Of the descriptions, 366 are a
single line with a single number (for example `"+165% Damage"`), which a regex plus `UpgradeTypes` can turn into
data. 74 are conditional (`On Kill`, `Stacks up to`, `for Ns`), and 246 have some other multi-line or multi-number
form. **Per-rank values are not stored.** They exist only as hand-written `emodtable` wikitext on each mod page
(`Serration.wiki`: rank 0 → +15% … rank 10 → +165%). The usual linear rule `value(r) = max × (r+1)/(MaxRank+1)`
reproduces Serration, but it is an assumption for other mods. Conclusion: use this module for *identity, polarity,
drain, compatibility and exclusion*. Effects need another source (DE export `levelStats`, or WFCD) or
hand-authoring.

Rivens: entries exist only as item stubs (`"Rifle Riven Mod"`, `MaxRank = 0`, `Class = "Riven"`). Riven stat base
values and the grade formula are wikitext on the `Riven Mods` page. Disposition is now the constant 1.2 (see §3).

Archon shards: **no module.** The only matches for "Archon" in modules are the `Archon *` warframe mods (Archon
Continuity, etc.).

## 5. Arcanes (`Module:Arcane/data`)

```lua
["Primary Merciless"] = {
    Description = "On Kill:\r\n+30% Damage for 4s. Stacks up to 12x.\r\n+30% Reload Speed",
    Dissolution = 20, InternalName = "/Lotus/Upgrades/CosmeticEnhancers/Offensive/PrimaryDamageOnKill",
    IsRefreshable = true, MaxRank = 5, Name = "Primary Merciless", Rarity = "Rare", Type = "Primary",
    UpgradeTypes = { "WEAPON_DAMAGE_AMOUNT", "WEAPON_RELOAD_SPEED" }
},
```

166 arcanes. Types: Warframe 64, Operator 22, Secondary 18, Primary 14, Amp 13, Melee 12, Zaw 8, Kitgun 8,
Tektolyst Artifacts 5, Bow 1, Shotgun 1. As with mods, the effect is max-rank prose. `UpgradeTypes` is on 92 of 166
and is sometimes misleading: Arcane Avenger and Arcane Fury both list `GAMEPLAY_FREE_REVIVE_COUNT`. Melee Duplicate
has no `UpgradeTypes` at all.

## 6. Enemies (`Module:Enemies/data/<faction>`)

```lua
["Heavy Gunner"] = {
    General = {
        Description = "High damage minigun", Faction = "Grineer", Image = "HeavyGunnerDE.png",
        InternalName = "/Lotus/Types/Enemies/Grineer/AIWeek/MinigunBombard",
        Link = "Heavy Gunner", Name = "Heavy Gunner", Scans = 3, Type = "Heavy Ranged",
        Weapons = { "Gorgon", "Sheev" },
    },
    Stats = {
        Attacks = { { AttackName = "Shot Damage",
                      DamageDistribution = { Impact = 0.75, Puncture = 0.15, Slash = 0.1 },
                      TotalDamage = 8, StatusChance = 0.05 } },
        Health = 300, EximusHealth = 300, Armor = 500, Affinity = 500, BaseLevel = 8,
        Multis = { "Head: 3.0x" },
    }
},
```

Counts per sub-module: grineer 281, corpus 352, infestation 86, orokin 32, sentient 33, stalker 9, narmer 42,
themurmur 27, techrot 13, scaldra 11, anarchs 26, unaffiliated 108. That is **1019 unique entries** (1020 records;
`Death Scuttler` appears twice). For comparison, the inspiration's `enemies-min.json` has 498.

| Field | Non-empty count | Notes |
|---|---|---|
| `General.Faction` | 1020 | 22 values. They include sub-factions that map to separate damage tables: `Kuva Grineer` 29, `Corpus Amalgam` 44, `Infested Deimos` 28, plus `Predator`/`Prey`/`Neutral`/`Unknown`/`Duviri`/`Tenno`/`Objects`/`?`. `General.FactionDamageOverride` (152) overrides which damage table applies (Thrax Centurion: Faction `Unknown`, override `Zariman`). |
| `Stats.Health` | 1015 | Integers at `BaseLevel`. 6 entries have neither health nor shield. |
| `Stats.BaseLevel` | 1013 | Missing on e.g. Archon Amar (has only `SpawnLevel = 150`). `SpawnLevel` on 338. |
| `Stats.Armor` | 664 | |
| `Stats.Shield` | 446 | |
| `Stats.Overguard` | 12 | Thrax Centurion `Overguard = 15`. The meaning is unclear: it may be a base value scaled by level or a multiplier (see Open questions). |
| `Stats.EximusHealth` / `EximusShield` / `EximusOverguard` / `EximusArmor` | 246 / 98 / 1 / 1 | One typo, `EximusSHield`. |
| `Stats.SteelPathHealthBonus` / `SteelPathShieldBonus` / `ArchimedeaHealthBonus` / `FactionScaling` | 14 / 6 / 3 / 3 | Rare special-case scaling. |
| `Stats.Multis` | 717 | **Prose strings**: `"Head: 3.0x"`, `"Fanny Pack: 3.0x"`, `"Head: +1 Crit Tier, 1.5x Critical Damage"`, `"Stealth/Finisher: 16.0x"`. 81 are the empty string. These need a small parser plus manual cases. |
| `Stats.ProcResists` | 71 | List of immune procs (Demolishers: `{ "Radiation" }`). |
| `Stats.Attacks` | 655 | Enemy outgoing damage. Not needed for DPS. |
| Damage reduction | **0** | No DR field anywhere. Eidolon Teralyst shows only `Health 50000, Shield 50000, Armor 125`. Boss and Archon DR must come from elsewhere. |

`Enemies/data/pre-U36` (849 entries, last edit 2024-06-27) is a frozen archive of the old health-class model. It
must not be used for current calculations.

## 7. Damage types and faction modifiers (`Module:DamageTypes/data`)

`Health` (34 entries) holds both the current faction tables (`Introduced = "36"`+) and legacy health classes
(`Cloned Flesh`, `Ferrite Armor`, `Alloy Armor`, `Proto Shield`, ...). Each entry has `Positives` / `Negatives` /
`Bypass` as `{ {DamageType, percent}, ... }` (excerpt after conversion):

```json
"Grineer":       {"Introduced": "36",   "Positives": [["Corrosive",50],["Impact",50]]},
"Kuva Grineer":  {"Introduced": "36",   "Positives": [["Corrosive",50],["Impact",50]], "Negatives": [["Heat",50]]},
"Orokin":        {"Introduced": "36",   "Positives": [["Puncture",50],["Viral",50]],   "Negatives": [["Radiation",50]]},
"The Murmur":    {"Introduced": "35.5", "Positives": [["Radiation",50],["Electricity",50]], "Negatives": [["Viral",50]]},
"Anarchs":       {"Introduced": "41",   "Positives": [["Impact",50],["Electricity",50]], "Negatives": [["Radiation",50]]},
"Overguard":     {"Introduced": "31.5", "Positives": [["Void",50]]},
"Shield":        {"Introduced": "11",   "Positives": [["Magnetic",75],["Cold",50],["Impact",50]],
                  "Negatives": [["Puncture",20],["Heat",50]], "Bypass": ["Toxin",["True",50]]}
```

Factions with a table: Grineer, Kuva Grineer, Corpus, Corpus Amalgam, Infested, Infested Deimos, Orokin, Sentient,
Narmer, The Murmur, Techrot, Scaldra, Anarchs, Zariman, plus Overguard, Shield, Armor, and Tenno tables. `Types` (20)
gives each damage type's `InternalName` (`DT_SLASH`), its proc (`ProcInternalName = "PT_BLEEDING"`) and a prose
`Status` summary. `Procs` (39) are descriptive only: no damage-per-tick, duration or stack numbers. The
**`Shield` entry looks legacy** (`Introduced = "11"`, contains the pre-U36 Magnetic/Cold/Impact bonuses) and must be
checked against the current `Damage` page before use. This module **runs code at load**: it builds `total` and
colour fields, so its output differs if you only parse the table literal (see §9).

## 8. Other modules

- **Stances/data** (132 stances, runs `addSharedCombos`): each combo has `Attacks = { {Dmg = {%...}, Hits = {...},
  Procs = {...}, Types = {...}} }`, sometimes with `Duration`. Example: Argent Scourge Slide `Dmg = {200}`,
  `Procs = {"Knockback"}`, `Duration = 0.78`. This gives per-hit damage multipliers for melee combo DPS. It has no
  timing for most combos, and timing is needed for DPS.
- **Warframes/data** (126 frames): base Health/Shield/Armor/Energy, polarities, passive as prose. Useful for
  frame-buff UI, not for numbers.
- **Ability/data** (339 abilities, prose) and **Ability/data/stats** (220 internal names → `{Label, Modifier, Values}`
  rows, e.g. `"Damage: |val1|", Modifier = "AVATAR_ABILITY_STRENGTH", Values = {Val1 = 500}`). Only partially
  covered: Rhino Roar has no stats entry. Too sparse for buff abilities without hand-authoring.
- **Focus/data**: 5 schools, 104 ways. Rank effects are prose (`Stats = "..."`).
- **Companions/data**: 41 companions.

## 9. Parsing in a Rust build tool — tested

Environment: no `lua`/`luajit` binary is installed (only `liblua5.4-0` as a library), no Python `lupa`, and
`cc`/`gcc` are present.

### PoC 1: `mlua` with vendored Lua 5.1 (recommended)

`research/external/cache/poc/lua2json/` (Cargo project, `src/main.rs`, about 100 lines):

- Dependencies: `mlua = { version = "0.10", features = ["lua51", "vendored", "error-send"] }` (resolved to
  0.10.5, `lua-src` 547.0.0), `serde_json` with `preserve_order`, and `anyhow`. A clean `cargo build --release`
  takes about 10–12 s, and the binary is 1.06 MB. `error-send` is needed to use `?` with `anyhow`.
- Lua 5.1 is chosen because Scribunto runs Lua 5.1 semantics.
- Sandbox: only the `math`, `string` and `table` libraries are loaded (no `io`, `os` or `package`). It defines stub
  globals `mw` (`loadData` returns `{}`, `site.namespaces[828].name = "Module"`) and `require` (returns an inert
  table), so modules that touch Scribunto APIs still load.
- Conversion: a table whose keys are exactly 1..n becomes a JSON array, and anything else becomes an object with
  sorted string keys. Integer-valued floats become JSON integers. `math.huge` becomes the string `"Infinity"`.
  Functions become `"<function>"`.
- Result: **all 37 modules tried converted successfully**: 8 weapon slots, Mods, Arcane, 13 enemy files, both
  Modular files, Warframes, Focus, Companions, DamageTypes, Stances, Ability, Ability/stats, Upgrades, Enum,
  Factions, itemCompatibility, Maximization and Flaicher/dispoData. `Modular/data/kitgun` (1.28 MB) converts in
  0.05 s. Output is in `research/external/cache/poc/out/*.json`.
- Run: `CARGO_TARGET_DIR=<somewhere> cargo build --release && lua2json in.lua out.json`.

### PoC 2: a table-literal parser with no execution (cross-check)

`research/external/cache/poc/minilua/luatable2json.py` (about 110 lines). It parses `return {…}` or
`local X = {…} … return X` without executing anything. It handles line and long comments, quoted strings with
escapes, long strings, numbers, `true`/`false`/`nil`, `math.huge`, `[k]=v`, `name=v`, and positional items.
Output is **identical** to the mlua JSON for 16 of 18 modules compared: all 8 weapon slots, Mods, Arcane, Enemies
grineer/corpus/unaffiliated, Modular, Modular/kitgun and Warframes. It **differs** on `DamageTypes/data` (the
computed `total` table is missing) and `Stances/data` (the parser picks up the `SHARED_COMBOS` local instead of the
stance table, because the module ends with `return addSharedCombos(StanceData)`). A Rust port of this parser
(using `nom`, or hand-written in about 200 lines) would cover every pure-data module, but it would have to
re-implement those two modules' logic.

Syntax census of the data modules: no long strings, no block comments in weapons/mods/enemies, no hex or exponent
numbers, and no `;` separators. The only non-literal token is `math.huge`, used 43 times in weapon modules. Line
comments do appear, including inside records (`--Introduced = "?"` in enemies).

### Recommendation

Use `mlua` (vendored Lua 5.1) in the data build tool (`build.rs` or a separate `xtask`, not the runtime engine):

1. Fetch the needed modules with batched `prop=revisions&rvslots=main` (≤50 titles per call, `maxlag=5`, a
   descriptive User-Agent). Store each module's `revid` and timestamp in the bundle manifest.
2. Evaluate each module in a sandboxed Lua 5.1 with `mw`/`require` stubs, and convert to `serde_json::Value`.
   Handle `math.huge`, and treat empty tables as "empty" (`{}` → `[]`: the converter cannot tell an empty map from
   an empty list; for example, `Grineer Queens` has `Stats = {}`).
3. Deserialize into typed Rust structs with `#[serde(default)]`, normalize typos (`Hitscan`, `IsSlilent`,
   `Ammocost`, `ForcedProc`, `EximusSHield`), and skip `_IgnoreEntry`.
4. Join to DE Public Export / WFCD on `InternalName` for the things this source lacks.

This keeps the C toolchain dependency out of the shipped binary if the conversion runs as a separate build step.
Vendored `lua-src` needs only `cc`.

## 10. Coverage against engine needs

| Engine need | Wiki modules | Notes |
|---|---|---|
| Weapon per-attack damage map, crit, status, fire rate, multishot | **Yes** (1984 attacks) | Multishot defaults to 1 when absent |
| Magazine, reload, ammo | Yes (all guns) | Battery reload fields too |
| Charge time / burst / spool | Yes (158 / 62 / 38) | `FireRate = math.huge` on Lanka |
| Falloff / AoE radius | Yes (1043 `Falloff`) | AoE radius = `Range` / `Falloff.EndRange` |
| Forced procs | Yes (716) | |
| Trigger type | Yes, as free text | Needs normalization (33 strings) |
| Incarnon form | **Yes** (as attacks) | |
| Incarnon evolutions | **No** | Wikitext only |
| Lich/Tenet bonus element | **No** (flag only) | |
| Beam chain / punch-through | Punch-through yes; chain no | |
| Kitgun parts | **Yes** (+1200 pre-built combos) | |
| Zaw parts | Yes (not pre-built) | |
| Mod identity/compat/polarity/drain/exclusion | **Yes** | |
| Mod effects per rank, machine-usable | **No** (max-rank prose + `UpgradeTypes`) | |
| Arcane effects | Prose only | |
| Archon shards | **No** | |
| Riven | Disposition (constant 1.2) only | Stat table is wikitext |
| Enemy faction / base level / HP / shield / armor | **Yes** (≈1013–1019) | |
| Enemy overguard | Sparse (12) | |
| Enemy special damage reduction | **No** | |
| Body-part multipliers | Yes, as prose (`Multis`) | |
| Faction damage modifiers | **Yes** (`DamageTypes/data`) | Shield entry looks legacy |
| Status DoT/duration numbers | **No** | |
| Stance combo multipliers | Yes (no timings) | |

## 11. Data-quality notes (with examples)

- Field typos: `Hitscan` ×2 (Zylok and Zylok Prime Incarnon), `IsSlilent` (Unarmed), `Ammocost` (Coda Bassocyst),
  `ForcedProc` singular (Shraksun Scaffold ×2), `EximusSHield` ×1, `Tilesets`/`Tileset` vs `TileSets`,
  `ExcludeFromSimulacrum` vs `ExcludedFromSimulacrum`, and a stray `General["1"]`.
- `AttackParentIndex` is applied inconsistently (Ack & Brunt slams → 2; Nikana Prime slams → none).
- Missing `ShotType` on 12 gun attacks and missing `Multishot` on 11 gun attacks.
- Missing `Polarities` on 74 guns and missing `CompatibilityTags` on 72 guns. It is unknown whether absence means
  "none".
- Archon Amar has no `BaseLevel`. Eidolon Teralyst has no DR. `Overguard` values are unexplained (Thrax 15).
- One enemy name is duplicated across faction files (`Death Scuttler`).
- Stale modules to ignore: `Enemies/data/pre-U36` (2024-06), `Flaicher/dispoData` (2020), `Weapons/data/credits`
  (2022), and the `_RetiredDisposition` values (pre-H44.1.3).
- Mod `UpgradeTypes` can be wrong or missing for conditional effects (Arcane Avenger → `GAMEPLAY_FREE_REVIVE_COUNT`;
  Melee Duplicate → none).

## Open questions

- Melee `HeavyAttack` is a single total. Does its type split follow the Normal attack proportionally? Check against
  the wiki `Heavy Attack` page or DE export.
- `Overguard` on enemies (12 entries, e.g. Thrax Centurion 15): is it a base value at `BaseLevel`, a multiplier, or
  something else?
- Is `DamageTypes/data` `Health.Shield` (Magnetic +75%, Cold/Impact +50%, Puncture −20%, Heat −50%) still correct
  after U36, or is it legacy? Cross-check with the `Damage` page (see `research/findings/wiki-core-damage-formula.md`).
- `ExtraHeadshotDmg` semantics: additive to the headshot multiplier, or a separate bonus? Negative on Alternox (−2).
- Does missing `Polarities` / `CompatibilityTags` mean "none" or "not filled in"?
- Can a 50-title batched API call return all modules without hitting the API result-size limit (about 6 MB of
  content)? Not tested; the fallback is per-module `?action=raw` (under 60 requests).
- Linear per-rank scaling of mod values (`max × (r+1)/(MaxRank+1)`) holds for Serration. It is not verified for
  mods with non-linear rank tables. A per-rank source is needed.
- Is the CC BY-NC-SA 3.0 license acceptable for the bundled data file the user ships? It requires attribution,
  non-commercial use, and share-alike.
