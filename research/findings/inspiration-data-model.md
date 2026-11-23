Status: draft

# Inspiration data model (warframe-damage.com)

Scope: the data structures the inspiration's client bundle reads (mods, weapons, attacks, enemies, rivens, arcanes, shards, buffs, element and faction tables) and, for every mod "action" key, the engine code that consumes it. All line numbers refer to `research/external/inspiration/min.pretty.js` (abbreviated `L1234`, function name in parentheses). Samples refer to `research/external/inspiration/samples/`.

Method: the `modsProps` array (L2-L5434) was parsed with Node (pure data literal, no code) and tallied; every action key was grepped in L5435-end and each hit mapped to its enclosing top-level function. Formulas below are transcriptions of the minified code; the damage pipeline itself is documented by other findings files, so formulas here are given only where they define what a key *means*.

---

## 1. Mods (`modsProps`, L2-L5434)

593 entries. `modsProps` is an array of plain objects. The riven is not in the array: `showRiven` writes it into the sparse index `modsProps[9999]` (L7273-L7341).

### 1.1 Top-level mod fields

| Field | Count | Type | Meaning | Consumers |
|---|---|---|---|---|
| `name` | 593 | string | Unique key, also the localisation key (`gL`, L7405) and the identity used in build links (`d.m[slot] = name`, L9690-L9698). | `findByName` L8035-L8038; everywhere |
| `img` | 593 | string | Image file name; fetched through `/getImg.php` (L7232, L7514). | UI only |
| `tags` | 593 | string[] | Compatibility tags. A mod is offered when `tags ∩ weapon.categories ≠ ∅` or `tags ∩ weapon.compTags ≠ ∅` (initMods filter L7136-L7230; `removeUncompMods` L8342-L8349; optimizer `wa` L16345-L16353 and `Nc` L16168-L16199). See 1.4 for counts. | |
| `action` | 593 | object (or `[]`) | The stat effects. Copied into the panel as `actions` (note the plural) by `showMod` L7527-L7536 and the optimizer L15549-L15558. | see 1.3 |
| `type` | 217 | string, absent = normal weapon mod | Which slot family accepts the mod: absent → main slots 0-7; `exilus` (36) → slot 8 or slots 0-7, `stance` (65), `weapon_mist` = weapon arcane (42), `frame_mist` = warframe arcane (17), `aura_mod` (12), `frame_mod` = warframe mod (27), `shard` = archon shard (18). | initMods L7136-L7230; optimizer `Nc` L16168-L16199 |
| `uncomp` | 126 | string[] | Names of mods that cannot be equipped together with this one (Serration vs Amalgam Serration, Galvanized vs base mod). | `isModCompatible` L6987-L6997; optimizer L16145 |
| `uncomptag` | 11 | string[] | Weapon compTags that forbid the mod: `POWER_WEAPON` ×10, `CROSSBOW` ×2, `GRNBOW`, `CRPBOW`, `INFBOW`, `OMICRUS` ×1 each. Mod is hidden if `uncomptag ∩ weapon.compTags ≠ ∅`. | L7144, L7154, L7223, `removeUncompMods` L8349-L8351, optimizer L16174, L16353 |
| `trigger` | 3 | `"Semi"` | Mod only allowed on weapons whose `trigger == "Semi"` (the three Cannonade mods). | initMods L7201-L7205; optimizer L16191-L16193 |
| `prefer` | 5 | number (percent) | Optimizer-only bias: score multiplied by `1 + prefer/100`, clamped to ±25 (`nb` L16355-L16368, `Vc` L16369-L16375). Values: Magnum Force −1, Heavy Caliber −1, Secondary Dexterity −1, Primary Dexterity −1, Gilded Truth +3. | optimizer L16383, L17657, L18792, L18986-L19006 |
| `onkill` | 29 | `true` | Effect needs kills. Optimizer only: option "no on-kill" excludes mods whose whole effect is in `WITH_COND` (`od` L16507, `Jd` L16510-L16516, `jd` L16236-L16240). Not read by the damage engine. | optimizer |
| `set` | 2 | `{ "1": 1, "2": 1.25, name: "Sacrificial" }` | Set bonus: every numeric action of a set mod is multiplied by `set[count of equipped mods with same set.name]`. | `getCountSetMods` L8514-L8518; `rebuildModStatsCache` L8713-L8715; `findAllBaseModsDmg` L8450-L8454; `findAllFactionMods` L8496; `getX2CritChanceForHeavy` L8815-L8820 |
| `info` | 77 | string | Tooltip text (arcanes, shards). Optimizer regex-classifies it: `/next shot|next attack|first shot after/i` → "next_shot", `/on ability cast|on casting|after casting/i` → "on_ability" (L21868-L21869, used L16258-L16262). | L7238, L7519, L7535 |
| `shardType` | 18 | `crimson`/`violet`/`topaz`/`emerald` (azure/amber exist in the counter but no weapon shard uses them) | Shard colour; counted for the violet electricity bonus. Copied to panel as `shard`. | `getSummElementsFromMods` L8549-L8604; showMod L7534 |
| `localeName` | runtime | string | Added by `modsWithLocale` (L7027-L7031). | UI |

Panel entry shape (what the engine actually reads, `modsPanel[slot]`, L7527-L7539): `{ name, actions, tags, uncomp, uncomptag, shard, info, set? }`. Incarnon perks put `{ name: "Incarnon", mod_id, prop_id, actions }` into slots 18-22 (`showIncMod` L7566-L7580, optimizer L15663-L15669). Empty slots are the number `0`.

### 1.2 Slot map (`modsPanel`, 30 entries, L5449-L5452)

From `index.html` data-id attributes and the optimizer table `Ba` (L21814-L21823):

| Slots | Content | Filter (initMods L7136-L7230) |
|---|---|---|
| 0-7 | weapon mods | untyped or `exilus`, `trigger` rule, tag match, `uncomp`, `uncomptag`, not already added |
| 8 | exilus | `type=="exilus"` |
| 9 | stance | `type=="stance"` and tag ∈ `compTags` only (not categories) |
| 10 | weapon arcane (non-archgun) | `type=="weapon_mist"`, tag ∈ categories |
| 11-12 | warframe arcanes | `type=="frame_mist"`, tag ∈ categories |
| 13-16 | warframe mods | `type=="frame_mod"`, tag ∈ categories or compTags |
| 17 | aura | `type=="aura_mod"`, tag ∈ categories |
| 18-22 | incarnon evolution perks | one perk per tier key of `weapon.incarnon[0]` |
| 23-27 | archon shards | `type=="shard"` |
| 28 | archgun arcane 1 | `weapon_mist` with tag `primary` |
| 29 | archgun arcane 2 | `weapon_mist` with tag `secondary` or a category match |

Archguns hide slot 10 and show 28/29 (`setWeaponInfo` L7787-L7801).

### 1.3 How actions are aggregated

`rebuildModStatsCache` (L8668-L8744) builds `modStatsCache.sums` (read by `getStatValFromMods`, L8745-L8748):

```
for each non-empty slot q with actions k:
  t = 1; if q.set: t = q.set[getCountSetMods(q.set.name)]
  if isMaxCond and k.WITH_COND:
      if k.WITH_COND.FOR_WEAPON: d(sums, k.WITH_COND.FOR_WEAPON, k.WITH_COND, t)
      else                       g(sums, k.WITH_COND, k, t)
  else if k.FOR_WEAPON:          d(sums, k.FOR_WEAPON, k, t)
  else                           g(sums, k, {}, t)
g(h, primary, fallback, t): add t*primary[key] for every numeric key; add t*fallback[key] only
                             for numeric keys NOT present in primary          (L8689-L8695)
d(h, forWeapon, base, t):   for each category c of the weapon: add t*forWeapon[c][key];
                             then add t*base[key] for numeric keys not overridden (L8675-L8688)
```

Consequences for our schema:

- `WITH_COND` values **replace** same-named base values, they are not added. Galvanized Chamber `{multishot: 0.8, WITH_COND: {multishot: 2.3}}` gives 2.3 total with conditions on, 0.8 off. `WITH_COND` holds the fully stacked value.
- With conditions off (`isMaxCond=false`, option `optAllModsConds`, L12035-L12038) `WITH_COND` is ignored entirely (the object is non-numeric so `g` skips it).
- Non-numeric keys (`element`, `phys`, `SMITE`, `stances`, `unique`, object-valued incarnon keys) are not in `sums`; they have dedicated readers.
- `modStatsCache.elements` sums `element` (WITH_COND.element wins per element when `isMaxCond`) (L8721-L8738).
- `incForm`/`normForm` (L8722-L8724, function `p` L8696-L8708): per mod, first-found value among `[k.INCARNON, k.WITH_COND, k]` (resp. `k.NORMAL`) is added; read only by `getStatFromIncForm` (L8793-L8797) for `flat_critmult` and `flat_status_chance`. Note it ignores `isMaxCond` and sets.

Other readers walk `modsPanel` directly: `findAllBaseModsDmg` (L8430-L8461, honours WITH_COND and sets), `findAllFactionMods` (L8480-L8502), `getSummElementsFromMods` (L8549-L8605), `getListElementMods` (L8606-L8629, ignores WITH_COND), `getPhysMods` (L8630-L8640, ignores WITH_COND and sets), `getNonCritModProps` (L8468-L8479), `findOneModsStat` (L8798-L8809, returns the first slot's raw value), `flat_critmult_below_cchance` filters (L8277-L8285, L8862-L8871).

### 1.4 Mod counts

By `type`: untyped 376, stance 65, weapon_mist 42, exilus 36, frame_mod 27, shard 18, frame_mist 17, aura_mod 12 (total 593).

By tag (a mod can carry several tags):

| Tag | n | Tag | n | Tag | n |
|---|---|---|---|---|---|
| secondary | 130 | melee | 128 | primary-sniper | 78 |
| primary-archgun | 75 | primary-rifle | 70 | primary-shotgun | 69 |
| primary-bow | 67 | primary | 57 | melee-glaive | 7 |
| POWER_WEAPON | 4 | SWORDS_STANCE | 4 | SOBEK | 3 |
| CLAWS_STANCE, DAGGERS_STANCE, DUAL_DAGGERS_STANCE, DUAL_SWORDS_STANCE, FIST_STANCE, HEAVY_BLADE_STANCE, NIKANAS_STANCE, POLEARMS_STANCE | 3 each | | | | |
| DAIKYU, GLAIVES_STANCE, GUNBLADE_STANCE, HAMMERS_STANCE, IRON_STAFF_STANCE, MACHETES_STANCE, SCYTHES_STANCE, SPARRING_STANCE, STAVES_STANCE, SWORDS_AND_SHIELD_STANCE, TONFA_STANCE, VIPER, WARFAN_STANCE, WHIPS_STANCE | 2 each | | | | |

Tags with 1 mod each (weapon augments, unique stances, sub-categories): AKBRONCO_PRIME, AKJAGARA, AKSTILETTO_PRIME, ASH_STANCE, AX52, BARUUK, BASMU, BAYONET_STANCE, BLADESAW_STANCE, BLADE_AND_WHIP_STANCE, BRONCO, BURSTON_PRIME, CATABOLYST, CONVECTRIX, "DARK DAGGER", DESERT_WIND_STANCE, DREAD, "DUAL CLEAVERS", DUAL_KATANAS_STANCE, EMBOLIST, EXALTED_BLADE_STANCE, EXCALIBUR, FLUX, FURIS, GLAXION, GORGON, GRAKATA, GRINLOK, HARPAK, HATE, "HEAVY SCYTHE_STANCE", HEK, HEMA, HIGASA, HYSTERIA_STANCE, JAVLOK, JAW_SWORD, KUNAI, LATRON, LONG_KATANA_STANCE, MAGNUS, MARELOK, MIRE, NUNCHAKU_STANCE, OCUCOR, OGRIS, PARIS_PRIME, PENTA, RAPIER_STANCE, RIPKAS, SHADOW_CLAWS_STANCE, SKANA, SOMA_PRIME, SUPRA, TETRA, TIGRIS, TITANIA_STANCE, TNJETTURBINEPISTOL, TONKOR, VENATO, VESPER77, ZYLOK, dual-pistols, melee-dagger, melee-dualdagger, melee-polearm, melee-whip.

Lower-case tags are weapon `categories`; upper-case tags are `compTags` (weapon name for augments, `<CLASS>_STANCE` for stances, `POWER_WEAPON` for exalted/ability weapons).

### 1.5 Complete action-key table

Conventions: values are fractions (0.9 = +90 %) unless stated. "pp" = percentage points. `Σ` = sum over equipped mods via `getStatValFromMods`. Example mods are the first mod in the table that uses the key; `WC.` prefix = only appears inside `WITH_COND` for that example.

#### Structural keys

| Key | Meaning | Consumer lines | Example |
|---|---|---|---|
| `WITH_COND` | Conditional values; replace base values when "trigger all conditions" is on (1.3). | `rebuildModStatsCache` L8710-L8727; `findAllBaseModsDmg` L8455-L8457; `findAllFactionMods` L8498; `getSummElementsFromMods` L8555-L8567; `setForceProcs` L8848-L8849 (weapon unique); `getStatFromUnique` L8785; `getStatFromAttackUnique` L8779; optimizer `Mb` L16459-L16467 | Primary Acuity `{WITH_COND:{headshot_mult:5.25, crit_chance_weakp:3.5, set_mutishot_to_default:1}}` |
| `FOR_WEAPON` | `{category: {key: value}}` overrides for weapons having that category; only `primary-bow` is used, to double `charge_time` for bows. | `rebuildModStatsCache` L8675-L8688, L8716-L8721; `showRiven` L7317-L7321 | Critical Delay `FOR_WEAPON:{"primary-bow":{charge_time:-0.4}}` |
| `INCARNON` / `NORMAL` | Per-form override sub-objects read by `getStatFromIncForm`. No mod or sampled incarnon perk uses them (data-model hook only). | L8723-L8724 | none |
| `element` | `{Heat|Cold|Electricity|Toxin: v}` elemental mods; slot order drives combination. | `getSummElementsFromMods` L8549-L8605; `getListElementMods` L8606-L8629; `rebuildModStatsCache` L8727-L8737; `newElements` L12902-L13069; riven `addt:"element"` L14701-L14704 | Hellfire `{Heat:0.9}` |
| `phys` | `{Impact|Puncture|Slash: v}` physical mods: `dmg[type] *= 1 + Σ phys[type] × (1 + weapon.unique.physModsInc)`. Ignores WITH_COND and sets. | `getPhysMods` L8630-L8640; applied in `reCalcWithNonCritBase` L9241-L9243 and `reCalcWithTypeBase` L9395-L9399 | Rupture `{Impact:0.9}` |
| `SMITE` | `{Faction: v}` faction damage: `factionMult = 1 + Σ` for the enemy's faction, + Roar (see 4.5). Keys used: Corpus, Grineer, Kuva Grineer, Infested, Infested Deimos, Orokin, Sentient, Murmur. | `findAllFactionMods` L8480-L8502; `getFactionMod` L9104-L9109; optimizer L16230, L17169, L17511 | Smite Grineer `{Grineer:0.3, "Kuva Grineer":0.3}` |
| `stances` | `{comboName: {total, hits[], statuses[], heavy?}}`: combo duration in seconds at 1.0 attack speed, per-hit damage multipliers (strings, `Number()`-ed at L10593), forced status per hit (`""` = none). | `showMod` L7540-L7551 (combo dropdown); `getMeleeCountHits10s` L10137-L10216; `procMeleeComboAttackEffect` L12115-L12150; optimizer L15595 | Blind Justice |
| `unique` | `"excalibur"` / `"baruuk"`: Chromatic Blade / Reactive Storm. Adds an element dropdown; attack 0's whole damage becomes the chosen type and the mod's `status_chance` is multiplied by `strengthMult`. | `uniqueMods` L7422-L7468; `setChromaticBlade` L7351-L7373; `setReactiveStorm` L7374-L7396; `removeUniqueMods` L7397-L7404 | Chromatic Blade `{unique:"excalibur", status_chance:3}` |
| `na` | Explicit "no modelled effect" marker (value 0 or 1). | optimizer only L16215, L16795, L17654, L21802 | Parry `{na:0}` |

#### Damage

| Key | Meaning | Consumer lines | Example |
|---|---|---|---|
| `base` | Additive base-damage bonus: `baseMult = 1 + Σbase + slam_base(slams) + base_heavy(heavy) + unique.base + custom + basePerMagasize (+1.5×strength for strengthType "mesa")`. | `findAllBaseModsDmg` L8430-L8461; `BaseModsDmg` L8462-L8467 | Serration 1.65 |
| `base_heavy` | `base` that applies only to heavy attacks of melee weapons. | L8438-L8440 | Killing Blow 1.2 |
| `base_per_status` | Condition-Overload type: per distinct status on the enemy. Additive: `n×v + base_per_cold + baseMult`; multiplicative when attack `isCoMult` or option `is_MultiplicativeBasePerStatus`: `(1 + base_per_cold + n×v) × baseMult`. Skipped for AoE and slam attacks and when `isMaxCond` is off. | `reCalcWithNonCritBase` L9222-L9236; `reCalcWithTypeBase` L9370-L9395; `getModelQueueDmg` L10400-L10403 | Galvanized Aptitude 0.8 |
| `base_per_cold` | Constant added in the same place as `base_per_status` (Secondary Shiver). | L9232, L9365, L10403 | Secondary Shiver `WC.4.5` |
| `base_per_magasize` | Flag only: adds `0.33 × sqrt(moddedMagazine)` to `base` (value ignored). | `getBasePerMagasize` L10235-L10239; L8446 | Primary Plated Round `WC.1` |
| `multiple` | Separate multiplier `(1+Σ)` on direct damage and status damage. | L9250, L9366, L10465 (`zb` L10497-L10501) | Double Tap `WC.4` |
| `dbl_mult` | Separate multiplier `(1+Σ)` (Mecha Empowered). | L9157, L9253, L9370, L10444 | Mecha Empowered 1.5 |
| `dmgMultAgainstFrozen` | Separate multiplier (Melee Careen). | L9251, L9367, L10466 | Melee Careen 1.5 |
| `multDAIKYUBroadhead` | Separate multiplier (augment). | L9252, L9369, L10406 | Spring-Loaded Broadhead `WC.0.4` |
| `multSharpshot` | Separate multiplier; on AoE attacks applied in the queue model instead. | L9364, L10404-L10405 | Longbow Sharpshot `WC.3` |
| `finisherDmg` | Multiplier `(1+Σ)` on the `finisher` damage type and on attacks with `enableFinisherModsWithSlash`. | L9368, L10471-L10473 | Covert Lethality 1 |
| `mult_overguard` | Multiplier against overguard: `Cb = Σ` (not `1+Σ`) when > 0. | L10407-L10408 | Secondary Fortifier `WC.8` |
| `slam_mult` | Slam/heavy-slam multiplier `1+Σ` for attacks with `type` "s"/"hs". | L10504-L10508 | Nira's Contempt 1.5 |
| `f_multiple` / `l_multiple` | Multiplier on the first shot of a magazine / last shots (only if magazine > 6). | `createDmgQueue` L12519-L12524; L10775-L10776 | Charged Chamber `WC.0.4`; Synth Charge 2 |
| `glaive_explode` | Adds a glaive explosion worth 0.5 × hit damage. | L10370, L11376 | Volatile Quick Return `WC.1` |
| `dmgOnStatusEff` | Flat damage per elemental status proc, scaled by faction weakness and armour. | `calcDmgOnStatusEff` L10240-L10252; L10376, L10897-L10903 | Cascadia Empowered 750 |
| `electricityShardAbilityDmg` | Multiplier `1+Σ` (POWER_WEAPON shards). | L10389 | +10% Ability Damage… 0.1 |
| `flat_change_dmg` (incarnon) | Flat base damage added to every attack via `addDmgToBase` (scaled across damage types). | `reCalc` L9640-L9646; `addDmgToBase` L13193-L13206; `reCalcWithTypeBase` L9360-L9412; L10561 | Furis perk 28 / `WC.58` |
| `base_noncrit`, `base_uncrit_unstatus` (incarnon) | `{value, chance?, type: "ADD"|"MULT"}` bonus on non-crit (and non-status) hits: `baseMult = (1+base+ADD) × (1+MULT)`. | `getNonCritModProps` L8468-L8479; `BaseModsDmg` L8462-L8467; L10217-L10234 | Phenmor/Laetum "Devouring Attrition" `{value:20, chance:0.5, type:"MULT"}` |

#### Critical

| Key | Meaning | Consumer lines | Example |
|---|---|---|---|
| `crit_chance` | Additive % bonus to crit chance. `CC = ((base_cc + add_cc×100) × (1 + Σcrit_chance + zoom + custom + WPbonus + attackUnique + heavyX2 + combo×per_combo + headCritChance) × B + flat×100 + optFlatCrit) × (1 + mult_crit_chance)` | `getFinalWeaponCritChance` L8194-L8267 (formula L8231-L8266); `setAdditionalMods` L8857; L10525 | Point Strike |
| `crit_chance_weakp` | Headshot-only crit bonus; additive inside the bracket, or multiplicative `B = 1+v` if option `is_MultiplicativeWeakCC` or the attack has `isWPCritMult`. | L8201-L8210 | Primary Acuity `WC.3.5` |
| `crit_chance_per_combo` | `× (comboMultiplier − 1)` added to the crit bracket. | L8219-L8225; L10524 | Blood Rush `WC.0.4` |
| `flat_crit_chance` | Flat crit chance added after multiplication (`×100` pp). | L8231-L8238; L10533 | Arcane Avenger `WC.0.45` |
| `mult_crit_chance` | Final multiplier on crit chance. | L8244-L8249; L10529 | Deadly Maneuvers `WC.4` |
| `crit_mult` | Additive bonus to crit multiplier: `CM = ((base_cm + flat_critmult) × (1 + Σcrit_mult + custom + attackUnique + heavy_crit_mult) + crit_mult_add + punctureBonus) × (1 + custom crit_mult_mult)`. | `getFinalWeaponCritMulti` L8268-L8329; `setAdditionalMods` L8879-L8892 | Vital Sense |
| `crit_mult_add` | Flat crit multiplier added after the multiplication. | L8293-L8296; L8888-L8890 | Tenacious Bond `WC.1.2` |
| `heavy_crit_mult` | Added inside the CM bracket for heavy attacks. | L8270-L8273; L8878-L8880 | Dreamer's Wrath 0.32 |
| `double_crit` | Per-hit chance (×100 %) to roll one extra crit tier (+100 % crit chance for that hit). | `setCritDmgToHit` L9822-L9852 (L9829, then `p && (d += 100)` L8312); L10682 | Vigilante Armaments 0.05 |
| `incr_CM_by_punc_status` | Adds `min(50, puncture/total × statusChance × 0.1)` to CM. | L8297-L8309 | Melee Doughty 1 |
| `incr_CC_by_hit_with_reset6tier2` | Each hit adds `v` flat crit; resets after 6 tier ≥ 2 crits. | L10573, L10673-L10680 | Secondary Enervate 0.1 |
| `flat_critmult` (incarnon) | Added to base crit multiplier; per form via `getStatFromIncForm`. | L8274-L8276; L8846-L8848 | none sampled |
| `flat_critmult_below_cchance` (incarnon) | `{val, cond}`: +val crit multiplier when crit chance < cond (%). Reduce keeps the last slot's value only. | L8277-L8285; L8862-L8871 | Furis "Prelude of Might" `{val:3, cond:40}` |
| `flat_crit_with_status` (incarnon) | Hit crit multiplier × (1+v) when the enemy has fewer than 3 status stacks. | L10470, L10752-L10760 | Phenmor "Spiteful Defilement" 1 |

#### Status

| Key | Meaning | Consumer lines | Example |
|---|---|---|---|
| `status_chance` | Additive % bonus: `SC = ((base_sc + 100×(flat_status_chance + incUp + attackUnique.flat)) × (1 + Σstatus_chance + slashBonus + combo×by_combo + custom + h_status_chance(heavy) + attackUnique) + absolute + custom abs) × (1 + status_chance_mult + custom + 0.1×tauStacks)`; returns base SC unchanged if attack has `noIncrStatus`. | `getFinalStatusChance` L9888-L9977 | Galvanized Aptitude 0.8 |
| `absolute_status_chance` | Flat pp added after multiplication (value in pp, e.g. 30). | L9933-L9937, L9965-L9969 | Napalm Grenades 30 |
| `flat_status_chance` (incarnon) | Flat added to base SC before multiplication (×100). | L9895-L9898, L9918, L9950 | Furis "Elemental Balance" 0.28 |
| `status_chance_by_combo` | `× (combo − 1)` in the SC bracket. | L9903-L9909 | Weeping Wounds `WC.0.4` |
| `h_status_chance` / `h_status_damage` | Heavy-attack-only status chance / status damage. | L9927, L9959; L10499 | Empowered Blades 0.6/0.6 |
| `status_damage` | Status (DoT) damage multiplier: `zb = (1 + Σstatus_damage + custom + h_status_damage) × (1+multiple) × (1+dmgMultAgainstFrozen) × (1+vuln_status_damage)`. | L10463, L10497-L10502 | Rifle Elementalist |
| `status_damage_toxin`, `status_damage_gas` | Per-type DoT bonus (`Sb` map L10464 with Heat, Electricity, Slash from other sources). | L10449, L10462 | Emerald shard 0.3; Leaded Gas `WC.3` |
| `vuln_status_damage` | Extra status-damage multiplier. | L10468-L10469 | Burning Hate 1.2 |
| `status_duration` | `curr_duration = int(duration × (1 + Σ + unique))` for every entry of `statusDuration`. | `setAdditionalMods` L9085-L9093 | Lasting Sting |
| `incMaxStacks_Corrosion` | Raises Corrosive stack cap (only when the enemy has `maxProcStacks`, see 3). | `procStatusArray` L9985, L10015-L10016; L10582 | Emerald shard 2 |
| `corrosive_by_toxin` | Archon Continuity: each Toxin proc also adds a Corrosive proc. | L9983, L10035, L10037-L10040; L10445 | Archon Continuity 1 |
| `archon_vitality` | Archon Vitality: each Heat proc adds an extra Heat proc. | L9984, L10036, L10039-L10041; L10446 | Archon Vitality 1 |
| `random_status_after_status` | Secondary Encumber: per proc, chance (×100) to add one random status (disabled if weapon `unique.sagekPassive`). | L9979-L9983, L10021-L10028; L10580, L10608-L10617; `setForceProcs` L12865-L12876 | Secondary Encumber 0.24 |
| `onPunctureApplyCold` | On a Puncture proc add `v` Cold procs. | L10559, L10743-L10750 | Secondary Cryogenic 3 |
| `add_slash_on_impact` / `add_magnetic_on_impact` | Chance (×100) per Impact proc to add Slash / Magnetic; doubled for non-melee with fire rate < 2.5. | `setProcsITS` L9853-L9878 | Internal Bleeding 0.35; Magnetic Welt 0.35 |
| `add_slash` | Hunter Munitions. Value is NOT read; the engine looks for the mod by name and hard-codes `0.3 × min(CC,100)` on crits. | `hasHunterMunitions` L9879-L9881; L10761-L10765 | Hunter Munitions 0.3 |
| `debilitate` | Primary Debilitate: after 10 ticks of a combined element, adds one of its components. | L10447, L10622-L10640 | Primary Debilitate 1 |
| `mDuplicate` | Melee Duplicate: tier-1 crits spawn a duplicate hit. | L10448, L10685-L10719 | Melee Duplicate 1 |
| `nNapalm` | Nightwatch Napalm: extra heat damage only for an attack named "Rocket Explosion". | L10409-L10440, L10766-L10770 | Nightwatch Napalm 1 |
| `armor` | Corrosive Projection: enemy armour × (1 − Σ). | L10372, L10812 | Corrosive Projection 0.18 |

#### Elements added outside combination (newElements L12902-L13069)

All add `totalBaseDamage(attack) × value` of the named type to every attack. "S" suffix and `…ScaledByStrength` multiply by `strengthMult`.

| Key | Meaning | Consumer lines | Example |
|---|---|---|---|
| `addViral`, `addCorrosive`, `addRadiation`, `addMagnetic`, `addGas` | Combined element added directly (bypasses combination). `addViral` also gets Nourish (4.5). | L13020-L13053 | Biotic Rounds `addMagnetic:1.5` |
| `addBlastS`, `addCorrosiveS`, `addRadiationS` | Same, × strength. | L13030, L13038, L13062 | Thermal Transfer 0.75 |
| `addHeatNotCombinedScaledByStrength`, `addColdNotCombinedScaledByStrength`, `addElectricityNotCombinedScaledByStrength` | Base element added uncombined, × strength. | L12966-L13001 | Fireball Frenzy 1 |
| `heat_scaledByStrength`, `electricity_scaledByStrength` | Extra combinable Heat/Electricity % × strength. | `getStatValFromModElements` L8749-L8763 | Fireball Frenzy 1 |
| `electricityBonus` | Violet shard: per equipped violet electricity shard, adds `v × (crimson+azure+violet shard count)` to Electricity. | L8549-L8604 (L8564-L8582, L8603) | +30% Primary Electricity Damage 0.1 |
| `add_weakness_dmg` | Sentient Incision: at recalculation its `actions` are replaced by a faction-specific `add*` (Infested→addHeatNotCombined, Anarchs→addElectricityNotCombined, Narmer→addToxinNotCombined, Grineer/Kuva/Zariman/Scaldra→addCorrosive, Corpus/Amalgam/Techrot→addMagnetic, Orokin→addViral, Sentient/Murmur→addRadiation, Deimos→addGas; default Grineer). | `addWeaknessDmgType` L7469-L7497 | Sentient Incision 1.2 |

#### Weapon stats (setAdditionalMods L8843-L9094)

| Key | Meaning | Consumer lines | Example |
|---|---|---|---|
| `speed` | `speed = base × (1+Σspeed+custom+zoom+attackUnique) × (1+speed_mult)`, floored at 0. | L8919-L8931; burst delay L8936-L8952; `reCalcWithTypeBase` L9431-L9444 | Speed Trigger 0.6 |
| `set_speed_to_default` | Flag: fire rate reset to base (Cannonade). | L8932-L8934, L9444 | Semi-Rifle Cannonade 1 |
| `charge_time` | `charge = base / (1+Σ)`. | L8973-L8982 | Speed Trigger 0.6 |
| `multishot` | `(base + flat_multishot) × (1 + Σ + custom + unique)`. | L8983-L9004 | Split Chamber 0.9 |
| `set_mutishot_to_default` | Flag: multishot reset to base. | L8844, L8990, L9002 | Primary Acuity `WC.1` |
| `magazineSize` | `round((base + baseMagazineSize) × (1+Σ+custom+unique) + add_magazineSize)` unless weapon `disableMagazChange`. | L9006-L9018 | Wildfire |
| `add_magazineSize` (incarnon) | Flat rounds. | L9016 | Furis "Extended Volley" 25 |
| `reloadTime` | `reload = base / (1+Σ+custom+unique)`; battery weapons: `reloadDelay/(1+Σ) + mag/reloadRate`. | L9024-L9041; `createDmgQueue` L12280-L12287 | Pistol Elementalist |
| `reloadRate` | Battery recharge rate `/ (1+Σ)` (sic: divides a rate). | L9019-L9023 | Shell Rush 0.5 |
| `ammoCapacity` | `floor(base × (1+Σ+unique))` (or `set_ammoCapacity × …`). No damage effect. | L9042-L9052 | Ammo Drum 0.9 |
| `ammoEff` | Ammo efficiency; effective magazine `= mag / (1 − min(Σ, 0.999999))`; ignored on incarnon attacks. | `createDmgQueue` L12459-L12470 | Primary Crux `WC.0.6` |
| `punch_through`, `shot_speed`, `range` | Display stats (flat add / multiply / flat add). No damage effect except weapon-unique `ifMaxStatPuncture` range. | L8953-L8972, L9064-L9069 | Primed Shred 2.2 |
| `comboDuration` | `+ seconds`. Display only. | L9053-L9058 | Gladiator Rush 6 |
| `initialCombo` | Hits added to the starting combo counter (combo multiplier = `int(1 + hits/20)`). | `addTimelineMeleeCombo` L12158-L12181 | Ready Steel 24 |
| `melee_combo_eff` | Heavy-attack efficiency: fraction of combo kept after a heavy, clamped to [0, 0.9]. | `getCurrEfficiency` L12151-L12153 | Focus Energy 0.4 |
| `windUp` | `windUp = base / (1+Σ)`; added to heavy attack cycle time. | L9059-L9063; L10208 | Killing Blow |
| `headshot_mult` | `headShotMult = 3 + Σ + custom + zoom + unique` (L5456 initial 3). | L9070-L9076 | Primary Acuity `WC.5.25` |
| `mult_for_head` | `headshotIncreaseDmg = (1 + Σ + custom + zoom + unique) × (1 + mult_for_head_mult)`. | L9077-L9084 | Target Acquired |

#### Keys with no damage effect (optimizer bookkeeping only)

`accuracy`, `recoil`, `spread`, `zoom`, `d_sound`, `ammoCapacity`, `arch_range`, `bounce`, `heal`, `lifesteal`, `energy`, `mutator`, `purity`, `beam_length`, `blast_radius` are listed in optimizer table `Ic` (L21796-L21813); `Sc` (L21876-L21890) gives the reason text ("does not affect damage"), including `punch_through`, `shot_speed`, `range`, `aColdSpread`. `comboDurationP`, `comboIn` appear only in the optimizer's melee table `ne` (L21891-L21904). Examples: Stabilizer `recoil:-0.6`, Eagle Eye `zoom:0.4`, Hush `d_sound:1`, Rifle Ammo Mutation `mutator:0.5`, Quick Return `bounce`, Shivering Contagion `aColdSpread:1`, Corrupt Charge `comboDurationP:-0.5`.

#### Dead keys (present in data, read nowhere)

`ammoEffFull` (Skull Shots, Brain Storm), `crit_chance_slide` (Maiming Strike), `energy_reg` (Primary Exhilarate), `finisher_dmg` (Hunter's Bonesaw; note the engine key is `finisherDmg`), `jam` (Primary Obstruct), `mInfluence` (Melee Influence), `on10RadiationAOEDmg` (Secondary Irradiate), `sniper_combo_duration` (Harkonar Scope), `sobekAug` (Acid Shells). Grep of L5435-end finds no reader.

#### Engine keys read by `getStatValFromMods` but used by no mod

`addBlast`, `addCold`, `addColdNotCombined`, `addColdS`, `addElectricity`, `addElectricityNotCombined`, `addElectricityS`, `addGasS`, `addHeat`, `addHeatNotCombined`, `addHeatS`, `addMagneticS`, `addPunctureStatus`, `addToxinNotCombined`, `addToxinNotCombinedScaledByStrength`, `add_crit_chance`, `add_toxin`, `add_toxinS`, `baseMagazineSize`, `decr_armor_by_punc`, `disableBodyCrit`, `flat_multishot`, `headCritChance`, `impactToPuncture`, `mult_for_head_mult`, `radius`, `set_ammoCapacity`, `set_status_chance`, `slam_base`, `speed_mult`, `status_chance_mult`, `status_damage_electricity(_scaledByStrength)`, `status_damage_heat(_scaledByStrength)`. Some are reached through rivens (`addBlast`, see 4.1) or `addWeaknessDmgType` (`add*NotCombined`); the rest are hooks with no data (likely for removed or planned items). Our schema should still support them, since weapon `unique` objects use several (5.2).

---

## 2. Weapons (`/getWeapon.php` JSON)

Loaded by `showWeapon` (L8007-L8027) → `setWeaponInfo` (L7780-L8006) → `buildWeapon` (L6970-L6977), which deep-copies into `currWeapon` (modded, recomputed), `currWeaponInitial` (base, mutated by kitgun/zaw/progenitor/flat-damage edits) and `currInitialAttacks` (pristine attacks).

### 2.1 Weapon-level fields

Observed in the 10 samples (count in parentheses) plus fields the code reads that the samples lack (marked *code*).

| Field | Example | Meaning / consumer |
|---|---|---|
| `name` (10) | "Furis" | identity; name hard-codes: "Tenet Plinx" L10515, "Balefire Charger (Hildryn)" L12291, "Lanka" L12294, "Evensong" L12851, "Cedo"/"Cedo Prime" L12855 |
| `category` (10) | "Primary" | display (L7953) |
| `productCategory` (10) | "LongGuns", "Pistols", "Melee", "Archguns" | slot layout L7787-L7801; melee checks (`meleeTxt`="Melee", L5498) |
| `type` (10) | "Rifle", "Pistol" (melee samples also "Rifle"); *code*: "Kitgun", "Zaw Component", "Exalted Weapon" | kitgun/zaw UI L7877-L7938; exalted → ability combo L10378-L10387; reCalc L9645-L9655 |
| `trigger` (9) | "Auto", "Semi", "Held", "Charge" | Cannonade gating L7201 |
| `tags` (10) | ["Tenno","Incarnon"], ["Grineer","Kuva Lich"] | "Incarnon" → incarnon block L7984; "Tenet"/"Kuva Lich"/"Coda" → progenitor element L7940-L7944, L10425-L10427; kitgun: `tags[0]` is the grip category key (L8163, L14533) |
| `categories` (10) | ["weapon","primary","primary-rifle"] | mod compatibility; `FOR_WEAPON` keys; "primary-shotgun"/"secondary-shotgun" shotgun flag L10366-L10369; "primary-bow" fire-rate model L12290 |
| `compTags` (10) | ["PROJECTILE","AOE","ASSAULT_AMMO"], ["FURIS"], ["NIKANAS_STANCE"] | mod compatibility; `BEAM` → 0.5 ammo/shot (L12267, L12472); `BATTERY`, `POWER_WEAPON` (L12493-L12494, Madurai L9254); `compTags[0]` selects `default_MeleeStances` (L10143) |
| `attacks` (10) | array, see 2.2 | |
| `comb` (4) | [[0,1]], [[1,2],[3,4]], [[0,1,2]] | groups of attack indices that can be "combined" (impact + explosion); UI L7993-L8004 |
| `magazineSize` (9), `incMagazineSize` (3), `disableMagazChange` (1) | 35 / 280 / true | incarnon attacks use `incMagazineSize` (L12264, L12460); `disableMagazChange` freezes magazine (L9008) |
| `ammoCapacity` (7), `reloadTime` (9) | 96, 1.6 | L9024-L9052 |
| `multishot` (9) | 1 | weapon-level base multishot (attack-level `multishot` overrides, L12431-L12434) |
| `reloadRate`, `reloadDelay` | *code* | battery weapons L9019-L9041, L12280-L12287 |
| `hasInfiniteMagazine` | *code* | L12463 |
| `zoomProps` (6) | `[[]]` | last element's `Upgrades[]` of `{UpgradeType, Value}` applied when conditions on (`getStatFromZoomLevels` L9095-L9103) |
| `incarnon` (3) | see 2.3 | |
| `noise`, `masteryReq`, `releaseDate`, `vaultDate`, `description`, `imageName`, `equipTime`, `maxLevelCap`, `additional_spread` | | display only (or unused) |
| melee: `blockingAngle`, `comboDuration`, `followThrough`, `range`, `windUp`, `slamAttack`, `slideAttack`, `heavySlamAttack` | 55, 5, 0.7, 2.5, 0.5, 594… | `comboDuration`, `range`, `windUp` modded (L9053-L9069); others display |
| `maxHeavyCombo` | *code* | overrides combo counter (L8264, L8850) |
| `unique` | *code* | weapon passives, see 5.2 |
| `strengthType` | *code*: "simple", "excalibur", "mesa", "cyte" | ability-strength scaling: simple/excalibur → whole damage × strength (L10349-L10353); mesa → +1.5×strength base (L8466); cyte → 1.25×strength (L9354-L9357) |
| `additionalBuff` | *code* | flat damage multiplier `na` (L10356) |
| `damage_mult` | *code* | multiplier in `addDmgToBase` (L13194) |
| `chamberName`, `tagLoader` | *code* (kitgun) | chamber key into `kitgunGrips[...]`, loader category into `kitgunLoaders` (L14533-L14567) |
| `strikeName` | *code* (zaw) | key into `zawStrikes` (L14587, L14610) |

### 2.2 Attack fields

| Field | Example | Meaning / consumer |
|---|---|---|
| `name` | "Normal Attack", "Incarnon Form", "Slam" | `isIncarnonAttack` = name contains "incarnon" (L8790-L8792); name contains "slam" → slam rules (L9225, L9381, L10509); "Rocket Explosion" → Nightwatch Napalm |
| `damage` | `{impact:3, slash:3, puncture:14}` | lower-case type → base damage; total = `getCurrAttackBaseDmg` L8641-L8645 |
| `speed` | 10 | fire rate / attack speed (shots/s) |
| `crit_chance`, `status_chance` | 34, 18 | **percent** (not fraction) |
| `crit_mult` | 3 | multiplier |
| `multishot` | — | per-attack override |
| `shot_type` | "Hit-Scan", "Projectile", "AoE", "Discharge" | "AoE" excludes `base_per_status`, applies `multSharpshot` in queue (L9226, L9382, L10399) |
| `isBeam` | true | 0.5 ammo per shot and no multishot rolls (`procMultishotArray` L10050-L10053, L12257, L12430) |
| `isInc` | 1 | incarnon-form attack: uses `incMagazineSize`, no ammo efficiency (L12459-L12465) |
| `isCoMult` | true | `base_per_status` multiplicative for this attack (L9235, L9393) |
| `charge_time` | 1.2 | bows/charge weapons: cycle `1/(charge + reload)`; Lanka `1/charge` (L12289-L12296) |
| `burst_count`, `burst_delay` | *code* | effective rate `S / ((S−1)·delay + 1/speed)` (L12258-L12262, L12288) |
| `ammoCost` | number or `"all"` | ammo per shot (L12267-L12275, L12472-L12477) |
| `no_headshot_mult` | true | no headshot multiplier (AoE parts) (L8313, L9831, L10357-L10364) |
| `falloff` | `{start, end, reduction}` | **display only** (only reader L7841-L7842) |
| `radius`, `punch_through`, `shot_speed`, `flight` | | display / modded display |
| `sniperCombo` | true | sniper combo multiplier (default 1.5) applies (L10326-L10329) |
| `unique` | `{force_procs:["impact"]}`, `{crit_mult:0.35}` | attack passives, see 5.2 |
| `type` (melee) | "s" slam, "hs" heavy slam | slam rules (L10149, L10505, L12142) |
| `isHeavy` | true/false | explicit heavy flag; `false` blocks the global heavy toggle (`isNeedCalcHeavy` L13253-L13258) |
| `slide`, `slam` (melee "Normal Attack") | "396", `{damage, radial{damage, element, radius}}` | unused (no reader) |
| `noIncrStatus`, `isWPCritMult`, `enableFinisherModsWithSlash`, `cedoRnd` | *code* | status chance cannot be modded (L9911); WP crit multiplicative (L8207); finisher mods apply (L10471); Cedo random procs (L12855-L12863) |

### 2.3 Incarnon evolution structure

`weapon.incarnon` is a one-element array; `incarnon[0]` maps slot ids `"18"`…`"22"` (tiers) to arrays of perks. A perk is `{name, img, type?, action}`:

- `name`: full display text ("Stormburst: Increase Damage by +28. On hitting…").
- `type`: attack names the perk is relevant to (`["Normal Attack"]`, `[]` = all). Only the optimizer reads it (`dd` L19416-L19429); the damage engine applies every selected perk to every attack.
- `action`: a mod-action object (same key space, plus the incarnon-only keys `flat_change_dmg`, `add_magazineSize`, `flat_critmult`, `flat_critmult_below_cchance`, `flat_status_chance`, `flat_crit_with_status`, `base_noncrit`, `base_uncrit_unstatus`), or `[]` for perks with no modelled effect.
- Selection: `showIncMod(slot, propIndex)` L7566-L7580 sets `modsPanel[slot] = {name:"Incarnon", mod_id, prop_id, actions}`; build link stores `{mId, pId}` (L9691-L9696). Tier "18" is the "Incarnon Form" toggle with empty action.

### 2.4 Mod compatibility (isModCompatible and friends)

```
offer(mod, slot) = slotFilter(mod.type, slot)                                  (L7136-L7230)
               and (mod.tags ∩ weapon.categories ≠ ∅ or mod.tags ∩ weapon.compTags ≠ ∅)
               and not (mod.uncomptag ∩ weapon.compTags ≠ ∅)
               and (mod.trigger absent or weapon.trigger == mod.trigger)
               and not isModAlreadyAdded(mod.name, slot)     // any panel name CONTAINS mod.name (L6998-L7006)
               and isModCompatible(mod.name, slot)           // no panel mod lists mod.name in its uncomp (L6987-L6997)
```

The substring test in `isModAlreadyAdded` makes "Point Strike" unavailable while "Primed Point Strike" is equipped (implicit primed/base exclusion). Weapon change re-runs the tag tests (`removeUncompMods` L8342-L8385) and clears slots 18-22, 28-29.

### 2.5 Kitgun and Zaw tables (L5581-L6301)

- `zawStrikes` (L5581-L5593): `{strike: {speed: offset}}`, 11 strikes.
- `zawGrips` (L6151-L6218): `{grip: {damage: flat, speed: base speed, type: [categories]}}`; offered if `type ∩ weapon.categories` (`getCompatibleGrip` L8167-L8174).
- `zawLinks` (L6219-L6301): `{link: {damage, speed, crit_chance (pp), status_chance (pp)}}`.
- Zaw application (handlers L14570-L14623): `attack.speed = strike.speed + grip.speed + link.speed`; `addDmgToBase(grip.damage + link.damage)`; `crit/status = currInitialAttacks + link.*`.
- `kitgunGrips` (L5594-L6026): `{gripCategory: {grip: {chamber: {attackName: damageMap}}, speed, charge, range?}}`; categories `primary-shotgun`, `primary-rifle-projectile`, `primary-rifle-hitscan`, `primary-rifle-beam`, `secondary-shotgun`, `secondary-projectile`, `secondary-hitscan`, `secondary-beam`; chambers Catchmoon, Sporelacer, Tombfinger, Rattleguts, Gaze, Vermisplicer; 5 grips per category (40 grip entries).
- `kitgunLoadersMain` (L6027-L6048): `{loader: {crit_mult, reloadTime}}`, 20 loaders; `kitgunLoaders` (L6049-L6150): `{shotgun|projectile|hitscan|beam: {loader: {crit_chance, status_chance, magazineSize}}}`.
- Kitgun application (L14520-L14569): grip sets each attack's `damage`, `speed`, `charge_time`; loader sets `crit_mult` (+1 if chamber is Sporelacer), `reloadTime`, `crit_chance`, `status_chance`, `magazineSize`.

### 2.6 Default melee stances (`default_MeleeStances`, L6302-L6581)

Keyed by stance compTag (e.g. `POLEARMS_STANCE`) or by weapon name (e.g. "Quassus", "Jat Kusar", "Sampotes", "Arum Spinosa") with `{total, hits[], statuses[], heavy:{total, hits[], statuses[]}}`. Used when no stance mod is equipped and always for heavy attacks (L10140-L10184, L12115-L12141, L12170-L12175). Frame exalted stances (ASH_STANCE, ATLAS_STANCE, TITANIA_STANCE, KHORA_STANCE, GARA_STANCE) are `{total:1, hits:["1"]}`.

---

## 3. Enemies (`samples/enemies-min.json`, showEnemy L7687-L7778)

498 enemies, array of objects. All numeric stats are **strings**.

| Field | Count | Example | Meaning / consumer |
|---|---|---|---|
| `name` | 498 | "002-ER" | key |
| `faction` | 498 | "Grineer" | weakness table `typeOfFaction`, faction mods, level-scaling branch in `correctEnemyLvl` (L8058-L8094) |
| `health.val`, `shield.val`, `armor.val` | 498 | "1000" | base stats at `b_lvl` |
| `b_lvl` | 36 | 15 | base level (default 1); level input clamped to ≥ b_lvl (L7696-L7698) |
| `innateDR` | 32 | `{health:0.625, armor:0.625, shield:0}`, `{status:0.5, …}` | multiplicative damage reduction per layer; `status` reduces status damage (L10894); applied as `(1 − innateDR[layer])` throughout getModelQueueDmg |
| `elemRes` | 22 | `{cold:0.75, blast:0.75, magnetic:0.75, viral:0.75}`, `{heat:0.8}` | per-type resistance: damage × (1 − r) (L9110-L9118, L10339, L10891-L10893) |
| `immun.status` | 52 | ["all"], ["heat"], ["viral","corrosive","magnetic"] | status immunity list → `enemyImmunStatus` (L7710) |
| `eximusOff` | 110 | true | hides the Eximus checkbox |
| `maxProcStacks` | 14 | 4, 10 | cap on stacks of each status (`maxCountProc` L7711; used L10011-L10019) |
| `unique` | 114 | "demolisher", "acolytes", "eidolon", … | special-DR class (see quirk Q1) |
| `overguard.val` | 4 | "2", "10", "15", "22" | base overguard (eximus without it uses 12) (`getEnemyOverguard` L7624-L7636) |
| `showPartyCount` | 18 | true | health × `healthPartyMult {1:1, 2:1.5, 3:2, 4:3}` (L5445, L8041-L8042) |

Runtime fields added: `currHealth`, `currShield`, `currArmor`, `currOverguard` (L8101-L8136).

Faction values in the data (count): Corpus 154, Grineer 118, Infested 49, Narmer 33, Sentient 26, Kuva Grineer 25, Orokin 19, Stalker 9, Infested Deimos 8, Neutral 8, Corpus Amalgam 7, Anarchs 7, Scaldra 6, Techrot 6, Murmur 5, Tenno 4, "" 4, UNKNOWN 3, Infestation 2, Unaffiliated 2, Arbiters of Hexis 1, " Infested" 1, " Sentient" 1.

`unique` values (enemies): acolytes (7), amalgam (6), sentient (18), archon (3), empyrean-corpus (6), raknoid (3), amalgam-machinist, crewmans-rangers (19), saxum (3), jugulus, demolisher (17), condrix, eidolon (3), vomvalyst, hyekka (3), hemocyte, juggernaut (2), lich, lephantis, necramech, orphix, stalker, sister, bursa, tusk-thumper (3), prosecutor, prelate, treasurer, suzerain, demolisherNecramech, efervonTank, dedicant, obsolyte, babau. The DR dispatcher handles only demolisher, eidolon, acolytes, amalgam, amalgam-machinist, empyrean-corpus, jugulus, saxum, lephantis, hemocyte, orphix, bursa, suzerain, archon, demolisherNecramech (`swithTypeUniqueEnemyDR` L13142-L13192).

Status-immunity tallies: radiation 17, all 15, viral 11, heat 10, impact 5, toxin 3, magnetic 2, slash 2, electricity 2, blast 2, cold 2, corrosive 1.

### 3.1 Level scaling (correctEnemyLvl L8039-L8138, for the schema only)

Default curves with `x = level − b_lvl`: shield `1+0.02x^1.75` → `1+1.6x^0.75`, health `1+0.015x^2` → `1+(24√5/5)·√x`, armour `1+0.005x^1.75` → `1+0.4x^0.75`, blended by smoothstep `trans(70,80)` (L7611-L7616). Faction overrides (L8058-L8094): Infested/Infested Deimos, Corpus, Grineer/Kuva Grineer, Orokin, Techrot, Sentient/Neutral/Murmur. Eximus uses separate piecewise functions (`enemyEximusArmor/Health/Shields` L7637-L7686). Steel Path in `showEnemy` multiplies base health and shield by 2.5 (L7699-L7702). Armour cap 2700 (L8121). Other findings files own the formulas.

---

## 4. Rivens, arcanes, warframe-side items, shards, buffs

### 4.1 Riven (showRiven L7273-L7341, attribute table L14695-L14765)

Up to 4 lines, each `{stat key, addt, value}`; value input in percent (−9999…9999). Encoding into `modsProps[9999].action`:

| UI name | key | addt | stored value |
|---|---|---|---|
| Critical Chance / Critical Multiplier / Multishot / Status Chance / Damage | `crit_chance`, `crit_mult`, `multishot`, `status_chance`, `base` | — | v/100 |
| Fire Rate / Attack Speed | `speed` | — | v/100; also sets `charge_time = v/100` and `FOR_WEAPON["primary-bow"] = {speed: 2v/100, charge_time: 2v/100}` (L7311-L7322) |
| Cold, Heat, Electricity, Toxin | `element.<Elem>` | element | v/100 (max 3 elements offered, L14760-L14764) |
| Impact, Puncture, Slash | `phys.<Type>` | phys | v/100 |
| Viral, Blast, Gas, Magnetic, Radiation, Corrosive | `addViral`, `addBlast`, `addGas`, `addMagnetic`, `addRadiation`, `addCorrosive` | — | v/100 |
| Status Damage | `status_damage` | — | v/100 |
| Damage to Corpus / Grineer / Infested / Scaldra / Techrot | `SMITE.<Faction>` | SMITE | input is a multiplier: stored `v − 1`; Grineer also writes `Kuva Grineer`, Infested also writes `Infested Deimos` (L7291-L7304) |
| Weak Point Critical Chance | `crit_chance_weakp` | **SMITE** (sic) | v/100 → lands in `SMITE.crit_chance_weakp` (quirk Q4) |
| Ammo Maximum, Ammo Efficiency, Weak Point Damage, Magazine Capacity, Reload Speed, Shot Speed | `ammoCapacity`, `ammoEff`, `mult_for_head`, `magazineSize`, `reloadTime`, `shot_speed` | — | v/100 |
| Combo Duration, Initial Combo, Range, Punch Through | `comboDuration`, `initialCombo`, `range`, `punch_through` | — | raw v |
| Heavy Attack Efficiency, Zoom, Weapon Recoil, Status Duration | `melee_combo_eff`, `zoom`, `recoil`, `status_duration` | — | v/100 |

Melee rivens drop punch_through, shot_speed, reloadTime, magazineSize, ammoCapacity, multishot, ammoEff; guns drop range, melee_combo_eff, initialCombo, comboDuration (L14741-L14754). Riven tags: primary, secondary, melee, primary-archgun (any weapon). Build-link form: `r[1..4] = {t: text, p: key, a: addt, v: value}` (L9751-L9765). No disposition or riven-stat-range modelling: values are free input.

### 4.2 Arcanes

Weapon arcanes (`weapon_mist`, 42) and warframe arcanes (`frame_mist`, 17) are ordinary mods whose effect sits mostly in `WITH_COND` at max stacks, with an `info` tooltip. Full list with actions is reproducible from `modsProps`; notable non-standard ones: Secondary Encumber (`random_status_after_status`), Secondary Enervate (`incr_CC_by_hit_with_reset6tier2`), Melee Doughty, Primary Debilitate, Melee Duplicate, Melee Careen, Secondary Cryogenic, Cascadia Empowered (`dmgOnStatusEff 750`), Primary Plated Round, Secondary Fortifier, Longbow Sharpshot, Akimbo Slip Shot (`dual-pistols` tag), Melee Exposure (`addCorrosive 2.4`), Melee Assimilation (`base_heavy`). Dead-effect arcanes: Primary Obstruct, Primary Exhilarate, Melee Influence, Secondary Irradiate, Melee Vortex (`na`).

The optimizer marks some as unreliable (`ae` L21848-L21862): Mecha Empowered, Proton Snap, Arcane Blade Charger, Arcane Primary Charger, Arcane Arachne.

### 4.3 Warframe-side mods and auras

`frame_mod` (27): Reactive Storm, Chromatic Blade (`unique`), Venom Dose, Shock Trooper, Freeze Force, Fireball Frenzy, Thermal Transfer, Smite Infusion (strength-scaled elements), Enraged, Merulina Guardian, Champion's Blessing, Gladiator Aegis/Resolve/Finesse, Nira's Anguish/Hatred, Tek Collateral, Vigorous Swap, Vigilante Pursuit/Vigor, Smoke Shadow, Biting Frost, Reinforced Bond, Tenacious Bond, Primal Rage, Archon Continuity, Archon Vitality.
`aura_mod` (12): Ready Steel, Rifle Amp, Worthy Comradery, Pistol Amp, Steel Charge, Shotgun Amp, Dead Eye, Holster Amp, Swift Momentum, Corrosive Projection, Mecha Empowered, Empowered Blades.

### 4.4 Archon shards (`type:"shard"`, 18; slots 23-27; can be added repeatedly, `showMod` L7502)

| Name | shardType | tags | action |
|---|---|---|---|
| +25% / +37.5% Critical Damage | crimson | melee | `crit_mult` 0.25 / 0.375 |
| +25% / +37.5% Status Chance | crimson | primary, primary-archgun | `status_chance` 0.25 / 0.375 |
| +25% / +37.5% Critical Chance | crimson | secondary, primary-archgun | `crit_chance` 0.25 / 0.375 |
| +50% / +75% Critical Chance | topaz | secondary, primary-archgun | `WITH_COND.crit_chance` 0.5 / 0.75 |
| +50% / +75% Critical Damage | violet | melee | `crit_mult` 0.5 / 0.75 |
| Incr. max stacks of Corrosion +2 / +3 | emerald | all weapons | `incMaxStacks_Corrosion` 2 / 3 |
| Toxin Status +30% / +45% more damage | emerald | all weapons | `status_damage_toxin` 0.3 / 0.45 |
| +30% / +45% Primary Electricity Damage | violet | primary, primary-archgun | `element.Electricity` 0.3 / 0.45, `electricityBonus` 0.1 / 0.15 |
| +10% / +15% Ability Damage affected by Electricity | violet | POWER_WEAPON | `electricityShardAbilityDmg` 0.1 / 0.15 |

(The tauforged variants are the second value of each pair.)

### 4.5 Buffs and options (setAdditionalParams L12029-L12114; build link `o` L9699-L9718)

| Link key | Global | UI | Effect (consumer) |
|---|---|---|---|
| `hs` | `is_headShot` | Headshot/Weak point | headshot multipliers, WP crit (L8195-L8203, L10357-L10364) |
| `s` | `stealthBonus` | Stealth (melee) | `getStealthDmgBonus` L8330, L10826 |
| `m` | `isMaxCond` | Trigger all conditions (default on) | WITH_COND, zoom props, base_per_status |
| `h` | `is_Heavy` + `comboСounterVal` | Heavy attack, start counter | heavy rules |
| `md` | `is_MultiplicativeBasePerStatus` | "damage per status multiplicative (~Condition Overload)" | L9235, L9393 |
| `w` | `is_MultiplicativeWeakCC` | (no checkbox in index.html; `optMultiplicativeCC`) | L8206 |
| `v` | `externalVirus {is_set, stucks}` | External Virus stacks 1-10 | viral stacks L10782-L10800 |
| `r` | `rhinoBuff {is_set,val}` | Roar % | added to faction multiplier `getFactionMod` L9104-L9109 |
| `z` | `mirageBuff` | Eclipse % | × (1 + val/100) L9156, L9249, L10427 |
| `x` | `hakuXata` | Xata's Whisper % | void damage `getXataDmg` L13226-L13243 |
| `l` | `toxicLash` | Toxic Lash % | `toxicLashDmg` L13215-L13225 |
| `n` | `nourishBuff` | Nourish % | `addViral += val/100` L13022 |
| `j` | `optFlatCrit` | Flat crit chance (~Kullervo, Harrow) | pp added in CC formula L8195, L8266 |
| `d` | `decreaseArmorOpt` | External armour decrease % | armour × (1 − val/100) L7698, L8040, L8121-L8122 |
| `ms` | `manualCountOfStatuses` | statuses on enemy | L10757, L12016 |
| `g` | `optMadurai` | Madurai +30 % | ×1.3 on non-POWER_WEAPON (L9254-L9256, L9495) |
| `t` | `strengthMult` | Ability strength % | `1 + (v−100)/100` (L14488) |
| `sc` | `sniperComboСounterSettings` | Sniper combo multiplier (default 1.5) | L10326-L10329 |
| `ac` | `abilityComboСounterSettings` | Combo ability multiplier | exalted weapons L10378-L10387 |
| (not in link) | `comboСounterSettingsVal` | Light-attack combo multiplier | L8219-L8224, L9904-L9909 |

Custom stats (`customStats`, `getCustomStat` L13259-L13264; link `p.a…p.aa` L9719-L9748): base, base_per_status, crit_chance, mult_crit_chance, flat_crit_chance, crit_chance_weakp, crit_mult, crit_mult_add, crit_mult_mult, status_chance, status_chance_mult, absolute_status_chance, status_damage, vuln_status_damage, speed, multishot, magazineSize, reloadTime, mult_for_head, headshot_mult, initialCombo, abilityCombo, dmgVulnerability, heatInherit, flat_change_dmg, heatAdd. Inputs with `data-d="1"` are percentages (divided by 100).

Other link keys: `w` weapon name, `e` enemy name, `l` level, `sp` Steel Path, `ex` Eximus, `j`/`jv` progenitor element and value (25-60 %, L7940-L7962, `addPElement` L12885-L12901), `z{l,g}` zaw link/grip, `k{l,g}` kitgun loader/grip, `m{slot: name | {mId,pId}}`. Encoded as LZString `compressToEncodedURIComponent(JSON)` in `?p=…&n=1`; without `n` the payload is plain base64 (`getSettingsFromUrl` L9779-L9786). Example decoded link: `research/test-cases/user-example-furis.json`.

---

## 5. Weapon/attack `unique` objects (passives)

Sample data has only `force_procs` and `crit_mult`. Keys the code reads:

### 5.1 Readers

`getStatFromUnique(key)` (L8782-L8789) reads `currWeaponInitial.unique[key]` (WITH_COND wins when `isMaxCond`). `getStatFromAttackUnique(key, attack)` (L8776-L8781) reads `attack.unique[key]` the same way.

### 5.2 Keys

- Weapon `unique`: `addBlast`, `addCold`, `addColdNotCombinedScaledByStrength`, `addCorrosive`, `addElectricity`, `addElectricityNotCombinedScaledByStrength`, `addGas`, `addHeat`, `addHeatNotCombinedScaledByStrength`, `addMagnetic`, `addRadiation`, `addToxinNotCombinedScaledByStrength`, `addViral`, `add_toxin`, `ammoCapacity`, `ammoEff`, `base`, `comboDuration`, `critFlatChancePerStatuses` (flat crit per distinct status, L8230, L10522), `crit_after_status` (`{chance_inflict, chance_crit}`, L10442, L10674-L10678), `crit_chance`, `crit_chance_weakp`, `double_crit`, `flat_change_dmg_by_strength` (L9363, L9643, L10563), `flat_crit_chance`, `headCritChance`, `headshot_mult`, `initialCombo`, `magazineSize`, `mult_for_head`, `mult_for_head_mult`, `multishot`, `physModsInc` (IPS mod amplifier, L8631), `range`, `reloadRate`, `reloadTime`, `sagekPassive` (L9980, L10579-L10606, L12534), `status_damage_slash` (L10461), `status_duration`, `windUp`, `WITH_COND`.
- Attack `unique`: `force_procs` (array of status names added to every hit, `setForceProcs` L12845-L12878), `addColdNotCombined`, `addHeatNotCombined`, `addToxinNotCombined`, `ammoEff`, `base_per_status`, `crit_chance`, `crit_mult` (Rubico zoom 0.35/0.5), `crit_mult_add`, `magazineSizeMult` (Tenet Plinx, L10515-L10519), `punch_through`, `radius`, `reconifexHeat` (L10565, L10815), `shot_speed`, `speed`, `speed_mult`, `set_speed_to_default`, `flat_status_chance`, `status_chance`, `absolute_status_chance`, `status_damage_heat`, `ifMaxStatImpact` / `ifMaxStatPuncture` / `ifMaxStatSlash` (bonus `{speed|range|status_chance}` when that IPS type is the attack's largest, L9418-L9480), `WITH_COND`.

---

## 6. Element and faction tables (L5498-L5580, transcribed in full)

```
meleeTxt = "Melee"; health = "health"; armor = "armor"; shield = "shield"; overguard = "overguard"   (L5498-L5502)
puncture slash toxin blast viral heat cold electricity magnetic = their lower-case names            (L5518-L5527)

physDmgTypes = ["Impact", "Slash", "Puncture"]                                                      (L5528)
baseElements = ["Heat", "Electricity", "Toxin", "Cold"]                                             (L5529)
comboElements = ["Blast", "Corrosive", "Gas", "Magnetic", "Radiation", "Viral"]                     (L5530)
othersElements = ["Void", "Finisher", "Microwave", "Tau"]                                           (L5531)
comboElementsInclude = {                                                                            (L5532-L5543)
  Heat: [], Cold: [], Toxin: [], Electricity: [],
  Blast: ["Heat","Cold"], Corrosive: ["Electricity","Toxin"], Gas: ["Heat","Toxin"],
  Magnetic: ["Cold","Electricity"], Radiation: ["Heat","Electricity"], Viral: ["Toxin","Cold"] }
```

Pairwise combination (`getTypeComboDmg` L8519-L8548): Heat+Cold=Blast, Electricity+Toxin=Corrosive, Heat+Toxin=Gas, Cold+Electricity=Magnetic, Heat+Electricity=Radiation, Cold+Toxin=Viral; base+combo → base; equal → itself. Optimizer copy `ge` (L21911-L21918) lists the same six as `[combo, a, b]`.

`typeOfFaction` (multiplier per damage type; absent = 1) (L5503-L5517):

| Faction | Modifiers |
|---|---|
| Grineer | Corrosive 1.5, Impact 1.5 |
| Kuva Grineer | Corrosive 1.5, Impact 1.5, Heat 0.5 |
| Sentient | Corrosive 0.5, Cold 1.5, Magnetic 1.5, Radiation 1.5 |
| Corpus Amalgam | Electricity 1.5, Blast 0.5 |
| Murmur | Electricity 1.5, Radiation 1.5, Viral 0.5 |
| Anarchs | Electricity 1.5, Radiation 0.5, Impact 1.5 |
| Narmer | Toxin 1.5, Slash 1.5, Magnetic 0.5 |
| Corpus | Puncture 1.5, Magnetic 1.5 |
| Orokin | Puncture 1.5, Radiation 0.5, Viral 1.5 |
| Infested | Slash 1.5, Heat 1.5 |
| Infested Deimos | Blast 1.5, Gas 1.5, Viral 0.5 |
| Zariman | Void 1.5 |
| Scaldra | Toxin 0.5, Corrosive 1.5, Impact 1.5 |
| Techrot | Cold 0.5, Gas 1.5, Magnetic 1.5 |

Cross-check (wiki `Damage` page, Update 36 notes, cached `research/external/cache/wiki/Damage.wiki` L684-L692): Grineer Impact+Corrosive, Corpus Puncture+Magnetic, Infested Slash+Heat, Orokin Puncture+Viral / resist Radiation, Sentient Cold+Radiation / resist Corrosive, Narmer Slash+Toxin / resist Magnetic, Zariman Void, Murmur Electricity+Radiation / resist Viral — all match. The wiki excerpt does not give the magnitudes (±50 %) or the sub-factions; Sentient's extra Magnetic 1.5 and every sub-faction row (Kuva Grineer, Corpus Amalgam, Infested Deimos, Anarchs, Scaldra, Techrot) are UNVERIFIED here.

Lookup: `findTypeOfRes(type)` (L9205-L9211) does a case-insensitive key match and returns `false` when absent; `calcPercentAdd(x, m)` (L8182-L8185) returns `x·m` if m > 1, `x − x·m` if 0 < m ≤ 1, else `x`; so a table value 0.5 gives 0.5x only because 1 − 0.5 = 0.5 (see quirk Q8).

Faction-mod baseline (`findAllFactionMods` L8481-L8492): `{Grineer, Sentient, Corpus, Orokin, Infested, Stalker, "Kuva Grineer", "Infested Deimos", Techrot, Scaldra}` all start at 1.

`statusDuration` (L5544-L5580; `duration` s, `ticks`, `max` stacks, `delay`):

| Status | duration | ticks | max | delay |
|---|---|---|---|---|
| Impact | 1 | 1 | 5 | 0 |
| Slash | 6 | 6 | ∞ | 1 |
| Puncture | 6 | 0 | 5 | 0 |
| Heat | 6 | 6 | ∞ | 1 |
| Electricity | 6 | 6 | ∞ | 0 |
| Toxin | 6 | 6 | ∞ | 1 |
| ToxicLash | 7.5 | 7.5 | ∞ | 1 |
| Cold | 6 | 0 | 10 | 0 |
| Blast | 1.5 | 0 | ∞ | 1 |
| Corrosive | 8 | 0 | 10 | 0 |
| Gas | 6 | 6 | 10 | 0 |
| Magnetic | 6 | 0 | 10 | 0 |
| Radiation | 12 | 0 | 10 | 0 |
| Viral | 6 | 0 | 10 | 0 |
| Void | 3 | 0 | 1 | 0 |
| Microwave | 1 | 0 | 1 | 0 |
| Tau | 8 | 0 | 10 | 0 |

(`curr_duration` starts equal to `duration` and is rescaled by status duration mods, L9088-L9093.)

Other globals (L5435-L5497): `healthPartyMult {1:1, 2:1.5, 3:2, 4:3}`, `headShotMultInitial = 3`, `combo_counterInitial = 11`, `comboСounterVal = 12`, `queueLength = 20` (simulated seconds), `enemyinnateDR` default `{armor:0, health:0, shield:0, overguard:0}`.

---

## 7. Implications for our schema (summary)

1. Mod = `{name, type/slot family, tags, incompatible names, forbidden compTags, trigger restriction, set {name, multipliers by count}, onkill, info, effects}`; effects need a base set and a conditional set with **replace** semantics, plus per-category overrides (`FOR_WEAPON`) and set multipliers.
2. Effect key space: ~140 keys (sections 1.5 and 5.2), many of them one-off mechanics (Hunter Munitions, Melee Duplicate, Debilitate, Encumber, Enervate, Doughty, Cryogenic, Fortifier, Napalm). We should model them as typed effects, not free-form keys, and drop or fix the dead keys.
3. Weapon = level fields + attacks with per-attack crit/status/speed/damage map, shot type, beam flag, incarnon flag, charge/burst/ammo-cost, headshot flag, passives; plus incarnon tiers, kitgun/zaw component tables, default stance tables.
4. Enemy = faction, base level, base health/shield/armor/overguard, innate DR per layer and for status, per-type resistances, status immunities, max stacks per status, special-DR class, Eximus/party flags.
5. Build state = 30 slots + riven lines + options + custom stats + progenitor element + kitgun/zaw picks; the inspiration's link format is reproducible from section 4.5 (useful for importing inspiration links).

---

## Suspected inspiration quirks

- Q1. `showEnemy` overwrites `currEnemy.unique` with `"notset"` (L7690), so `swithTypeUniqueEnemyDR` (L13142-L13192) and every `currEnemy.unique == …` branch in getModelQueueDmg (L10540-L10556) never fire: all special-enemy DR (Demolisher, Eidolon, Acolyte, Archon, Amalgam, Orphix, …) is effectively disabled in this build. The UI warns that unique DR "will not always be fully taken into account" (L7752).
- Q2. Faction mods vs Murmur: `findAllFactionMods` has no `Murmur` key (L8481-L8492), so `b.Murmur += v` gives `NaN` and `getFactionMod` falls back to 1 (L9106). Bane/Cleanse/Expel/Smite of the Murmur do nothing. Likewise no `Corpus Amalgam` key and no Corpus mod writes one, so Corpus faction mods do not apply to Corpus Amalgam enemies, while the wiki (Faction_Damage_Bonus, cached L8) says they do.
- Q3. Riven faction lines only offer Corpus, Grineer, Infested, Scaldra, Techrot (L14717-L14721); no Orokin/Sentient/Murmur, which the wiki lists as riven traits (Faction_Damage_Bonus L148).
- Q4. Riven "Weak Point Critical Chance" has `addt: "SMITE"` (L14722-L14725), so it is stored as `action.SMITE.crit_chance_weakp` and never reaches the crit code: the stat is silently ignored.
- Q5. Display text check in the riven handler compares `"Infected" != y` instead of "Infested" (L14792), so Infested faction lines show a stray "%".
- Q6. `burst_delay` is divided by `(1 + speed)` but then **multiplied** by `(1 + speed_mult)` (L8936-L8952); a multiplier on fire rate should divide.
- Q7. `reloadRate` mods divide the recharge rate (L9019-L9023) where faster recharge should increase the rate (or the field is a time, mis-named).
- Q8. `calcPercentAdd` (L8182-L8185) treats a modifier m ≤ 1 as a reduction by m (`x − x·m`): 0.5 → 0.5x works only because 1 − 0.5 = 0.5; any future 0.75 entry would give 0.25x. `findTypeOfRes` returning `false` relies on the third branch.
- Q9. Enemy armour: `0 < F ≤ 200` is forced to 200 (L8122). Any low-armour enemy at any level gets 200 armour (minus external decrease). Looks like a deliberate floor with no wiki source; UNVERIFIED.
- Q10. Steel Path multiplies base health and shield by 2.5 but not armour (L7699-L7702), and the external armour decrease is skipped in the SP branch (only the non-SP branch applies `d - d*p`, L7703-L7708).
- Q11. Slash DoT uses `V = (1 − innateDR[layer])²` (L10885, used at L10919-L10920): innate DR applied twice on Slash ticks.
- Q12. `getPhysMods` ignores `WITH_COND` and sets (L8630-L8640); `getListElementMods` ignores `WITH_COND` (L8606-L8629). No current mod is affected (checked: no mod has `phys` with `WITH_COND`), but the code path is inconsistent.
- Q13. `flat_critmult_below_cchance` reduce keeps only the last mod's value (L8279-L8284, L8865-L8870) instead of summing.
- Q14. `base_per_magasize` ignores its value and hard-codes `0.33·√magazine` (L10235-L10239).
- Q15. Hunter Munitions is found by name and its 30 % is hard-coded (L9879-L9881, L10761-L10765); the `add_slash` value is unused.
- Q16. `mult_overguard` multiplies by Σ, not 1 + Σ (L10407-L10408).
- Q17. Mod key typo/duplication: `finisher_dmg` (Hunter's Bonesaw, unused) vs engine `finisherDmg`; nine dead keys listed in 1.5.
- Q18. Data asymmetry in `uncomp`: Galvanized Acceleration excludes Fatal Acceleration but not the reverse; Argon Scope → Galvanized Scope not reciprocated; Primed Deadly Efficiency → Deadly Efficiency not reciprocated (masked by the substring rule); `uncomp` references non-existent mods "Amalgam Ripkas True Steel", "Amalgam Argonak Metal Auger", "Hydraulic Gauge", "Loose Hatch", "Maximum Capacity". Since `isModCompatible` checks only the equipped mods' lists (L6987-L6997), equipping Fatal Acceleration first then Galvanized Acceleration is allowed.
- Q19. `isModAlreadyAdded` is a substring test (L7001-L7003): any mod whose name is contained in an equipped mod's name is blocked (intended for Primed variants, but also hits unrelated pairs by accident).
- Q20. `falloff` is display-only (only reader L7841-L7842): damage falloff is never applied.
- Q21. Enemy data hygiene: factions `" Infested"`, `" Sentient"` (leading space), `""`, `UNKNOWN`, `Infestation` will miss every faction table lookup.
- Q22. Incarnon perk `type` (which attacks a perk applies to) is ignored by the damage engine; every selected perk applies to every attack (only the optimizer's `dd` reads it, L19416-L19429).
- Q23. `rebuildModStatsCache` incForm/normForm (L8696-L8708) ignores `isMaxCond`, so `WITH_COND.flat_status_chance` / `flat_critmult` would apply even with conditions off.
- Q24. Kitgun loader adds +1 crit multiplier when the chamber is Sporelacer (L14559-L14560): a hard-coded special case not in the tables.
- Q25. `getStatFromUnique` and `getStatFromAttackUnique` use truthiness (`d[b] &&`), so a deliberate 0 in `WITH_COND` cannot override a non-zero base value (L8776-L8789).

## Open questions

- OQ1. Is Q1 (`unique = "notset"`) intentional (unique DR deemed unreliable) or a leftover debug line? Its effect on the user's Furis-vs-Corrupted-Heavy-Gunner test case is nil (no `unique`), but it matters for any Demolisher/Eidolon oracle test.
- OQ2. Meaning of the `elemRes` 0.75 block on Corpus crewmen/rangers ("crewmans-rangers" unique; cold/blast/magnetic/viral 0.75) and the Hyekka heat 0.8: no wiki source found yet.
- OQ3. `headShotMultInitial = 3` (L5456) and the crit formula `p·(1 + tier·(2·CM − 1))` (L8319-L8328) imply headshot ×2 on top of a ×1.5 base? The exact intent needs the crit/headshot findings file.
- OQ4. `optMadurai` claims "physical damage +30 %" but is a flat ×1.3 on all damage of non-POWER weapons (L9254-L9256, L9495); intended or simplification?
- OQ5. Sub-faction weakness rows (Kuva Grineer heat 0.5, Corpus Amalgam, Infested Deimos, Anarchs, Scaldra, Techrot) and Sentient's Magnetic 1.5 need a wiki source.
- OQ6. `getEnemyOverguard` uses 12 base overguard for an Eximus without data (L7627-L7630) and `eOGTrans` calls `tXOG` on an already-normalised value (L7617-L7623); whether the blend is right is for the scaling findings file.
- OQ7. `additionalBuff`, `damage_mult`, `strengthType`, `hasInfiniteMagazine`, `reloadRate`/`reloadDelay`, `burst_count`, `ammoCost:"all"`, `maxHeavyCombo` and all weapon `unique` keys appear only in code; we have no sample weapon carrying them. Fetching more `/getWeapon.php` samples (e.g. a battery weapon, a burst weapon, an exalted weapon, a Sagek/Gotva/Reconifex) would confirm their units.
- OQ8. `glaive_explode` adds `(Jb/Ha)·g·0.5` (L11376): the variables were not traced; the exact base of the 50 % is unconfirmed.
