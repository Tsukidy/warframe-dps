Status: draft

# Inspiration (warframe-damage.com) feature inventory

Purpose: a complete list of every user-facing input, option, output and workflow of the inspiration, so the project can build a feature-parity checklist. Each entry gives what it does, its inputs (range / default), the code location and which calculation it feeds.

Sources and citation style:
- `JS L<a>-<b> fn` = `research/external/inspiration/min.pretty.js`, lines a-b, function `fn` (top-level names are real; locals are minified). Event handlers live in the `jQuery(function(){...})` block, JS L13810-15251. The optimizer is the `WFOpt` IIFE, JS L15252-22038.
- `HTML L<n>` = `research/external/inspiration/index.html`. `index.txt` / `guide.txt` are the rendered text of the main page and the building guide.
- Wiki pages are cached under `research/external/cache/wiki/` (Armor.wiki, The_Steel_Path.wiki were fetched for this file).

Mechanics are only described as far as needed to say *what a control changes*; the damage formulas themselves belong to the other research files.

---

## 1. Page layout and global chrome

| Feature | What it does | Code |
|---|---|---|
| Header: title link, "Building Guide" link (`/en/guide/`), "TOP 20 Monthly Weapon" button, Telegram link | Navigation. TOP 20 loads `/chart.js` and `/data/json/top-used.json` and draws a horizontal Chart.js bar chart in a popup. Server-only; out of scope per CLAUDE.md. | HTML L54-98; JS L15061-15109 (`.topUsed` click) |
| "Copy build url to clipboard" icon | Copies `document.location.href` (which always holds the current build link, see section 15) and alerts "Your build url has been copied to the clipboard." | HTML L66; JS L14910-14914 |
| Language dropdown | 7 entries: English (`/`), 简体中文 (`/zh/`), Русский (`/ru/`), 日本語 (`/ja/`), 한국어 (`/ko/`), Español (`/es/`), Français (`/fr/`). Each link is rewritten after every recalculation to carry the current build (`/<lang>/?p=...&n=1`). | HTML L77-92; JS L9770-9776 (inside `reCalc`) |
| "How does this work" | Popup `#queueInfo` explaining the random-queue algorithm (13 steps) and a list of known simplifications ("Calculation damage nuances"). | HTML L94-96, L512-557; JS L14919-14922 |
| Change log | Popup `#changeLogInfo` with dated entries (latest 2026.11.20). Useful as a list of mechanics the author added; see index.txt L209-430. | HTML L591-1056; JS L14915-14918 |
| Donate sticker / popup, cookie (Yandex Metrica) consent | Out of scope. | HTML L569-590, L1219-1355 |
| Popups in general | Close with the X, ESC, or a click outside the container; body scroll locked while open. | JS L13822-13826 (`d()`), L14940-14956 |
| Keyboard support | Arrow Up/Down move a highlighted row in the weapon, enemy and mod result lists; Enter picks it (or "clicks" the focused element); Escape closes the mod search; Delete removes the focused mod. Slots have `tabindex=0`. | JS L7032-7125 `setActiveItem`, `handleKeyPress` |
| Enemy-field reminder | After a weapon is picked but no enemy, the enemy input outline blinks blue/red every 700 ms. | JS L6960-6969 |

## 2. Weapon search and attack selection

- **Weapon search box** `#weaponSearch` ("Type or select weapon name"). On input or click it filters the inline list `Weapon` (666 names in HTML L46) by substring of the localized name (`searthLocaleTerm`) and highlights the match. JS L6583-6614 (`showList`), L6667-6671.
- **Picking a weapon** (`#resultsWeapon li` click) calls `showWeapon(name)`, which fetches `/getWeapon.php?name=<name>` (or uses the server-embedded `weapon_json` when the page was opened from a shared link for that weapon), then `setWeaponInfo`, then `reCalc`. JS L8007-8027 `showWeapon`, L14218-14226.
- `buildWeapon` deep-copies the weapon three ways (`currWeapon`, `currWeaponInitial`, `currInitialAttacks`) and calls `removeUncompMods`: **mods already installed stay installed when you switch weapon if they are still compatible**; incompatible ones, all Incarnon evolutions (slots 18-22) and archgun arcanes (28-29) are cleared. JS L6970-6977, L8342-8386.
- **Attack selection: there is none in the main UI.** Every entry of `weapon.attacks[]` (e.g. "Normal Attack", "Incarnon Form", "Rocket Impact"/"Rocket Explosion", melee "Slam"/"Heavy Slam") is simulated independently and gets its own DPS row in each result panel. The help text says attacks "are not aware of each other's existence" (index.txt L152-155). Only the optimizer has an Attack selector (section 17). JS L9132-9153 `reCalcDisplay`, L12422-12786 `createDmgQueue` loop over `currWeapon.attacks`.
- **Melee stance combo selector** `#stanceAttackList` (a `<select>`): appears when a stance mod is placed in slot 9; options are the keys of the stance's `action.stances` (combo names); label "*selected attack affects damage". The chosen combo's hit multipliers and forced statuses are applied per hit (`procMeleeComboAttackEffect`); without a stance a per-weapon-class default from `default_MeleeStances` is used. Changing it triggers `reCalc`. JS L7537-7547 (`showMod` slot 9), L12115-12150, L14517-14519; table JS L6302-6581. Not saved in the build link.
- **Heavy attack auto-mode**: attacks with an explicit `isHeavy` flag (Slam = false, Heavy Slam = true in samples/weapon-Nikana_Prime.json) ignore the "Heavy attack" checkbox (`isNeedCalcHeavy`, JS L13253-13258).

## 3. Weapon info panel (output)

Rendered by `setWeaponInfo` (JS L7780-8006) and updated live after every recalculation by `setInfoStat` (JS L8764-8775) and the per-type block in `reCalcWithTypeBase` (JS L9470-9486).

- Left block: localized name and category, description, "Mastery Required", image; additional props when present: multishot, ammo capacity, magazine size, **incarnon magazine size**, reload time, recharge delay, recharge rate (battery weapons), blocking angle, combo duration, follow through, range, wind up, release date, noise (JS L7856-7880).
- Per attack: name plus `(shot_type)`; **per-damage-type values and Total, shown after mods** (base x modded multipliers, `toFixed(1)`); critical chance, critical multiplier, status chance, multishot, flight, radius, shot speed, speed (fire rate / attack speed), charge time, ammo cost, punch through, falloff (start / reduction / end), burst count, burst delay, and "force procs" list. Modded values replace the base numbers in place (crit chance, crit mult, status chance, speed, burst delay, shot speed, radius, punch through, charge time, multishot, magazine size, reload rate/delay/time, ammo capacity, combo duration, wind up, range; JS L8843-9069 `setAdditionalMods`).
- This panel is the only "per damage type breakdown" in the UI. There is no DPS-by-damage-type table.

## 4. Weapon variants

### 4.1 Incarnon form and evolutions
- If the weapon has tag `Incarnon` and an `incarnon` array, an `.incarnonBlock` of slot tiles is inserted next to the mod grid, one tile per key of `incarnon[0]` (always ids 18-22 in the data: 18 = "Incarnon Form" tier, 19-22 = evolution tiers II-V). Clicking a tile opens a popup listing that tier's perks (name + image); choosing one stores `{name:"Incarnon", mod_id, prop_id, actions}` in `modsPanel[id]`. JS L8028-8034 `addIncBlocks`, L7566-7580 `showIncMod`, L14075-14096, L14174-14184.
- Perk effects are mod-like actions (e.g. `flat_change_dmg`, `add_magazineSize`, `flat_status_chance`, `flat_critmult_below_cchance`, `WITH_COND` variants; samples/weapon-Furis.json). Perk actions can be split into `INCARNON` / `NORMAL` sub-blocks so they apply only to the Incarnon-form attack or only to the normal attack (`modStatsCache.incForm` / `normForm`, JS L8668-8744 `rebuildModStatsCache`, L8790-8797). An attack counts as Incarnon if its name contains "incarnon" (JS L8790-8792) or it has `isInc` (JS L12459).
- The Incarnon Form attack is always listed as its own attack; selecting the tier-18 tile is not needed to see its DPS (UNVERIFIED by running; based on `createDmgQueue` iterating all attacks). Incarnon form uses `incMagazineSize` and ignores Ammo Efficiency (JS L12264-12266, L12459-12464).
- Removal: right-click / Delete / drag-out like mods; tile reverts to blank image. JS L8420-8428.

### 4.2 Kitguns
- For `type == "Kitgun"` two dropdowns: **Grip** (grips valid for the chamber's class `tags[0]`, from `kitgunGrips`) and **Loader** (20 loaders from `kitgunLoadersMain`). JS L7882-7908, L8154-8166.
- Grip sets each attack's damage table (per chamber and attack name), fire rate and charge time; Loader sets crit multiplier (+1 for Sporelacer), reload time, and from `kitgunLoaders[tagLoader]` crit chance, status chance and magazine size. JS L14520-14572; tables JS L6027-6150.
- Saved in the link as `k.g` / `k.l` (only when both are chosen).

### 4.3 Zaws
- For `type == "Zaw Component"` dropdowns **GRIP** (filtered by the strike's melee category) and **LINK** (all links). JS L7909-7934, L8167-8181.
- Grip and link add flat base damage (`addDmgToBase`) and attack-speed offsets on top of `zawStrikes[strikeName].speed`; link also adds crit and status chance. JS L14573-14624; tables JS L5583-5600 (`zawStrikes`), L6151-6301.
- Saved as `z.g` / `z.l`.

### 4.4 Progenitor element (Kuva / Tenet / Coda weapons)
- Shown when weapon tags include `Tenet`, `Kuva Lich` or `Coda`: element dropdown (Toxin, Cold, Heat, Magnetic, Electricity, Radiation, Impact) and a % input `#pElVal` (min 25, max 60, default 60; out-of-range values clamped). Calls `addPElement(elem, pct/100)`. JS L7935-7962, L14625-14654, `addPElement` L12885-12901.
- Saved as `j` (element) and `jv` (fraction, default 0.6 on restore).

### 4.5 Exalted-weapon unique mods
- Installing a mod with `action.unique == "excalibur"` (Chromatic Blade) adds a "Chromatic Blade element" dropdown (Heat default, Toxin, Cold, Electricity): all base damage of attack 0 is converted to that element and the mod's status chance is multiplied by ability strength. `"baruuk"` (Reactive Storm) adds "Reactive Storm weakness" (Impact, Slash, Puncture, 6 combined, 4 base; Radiation default). JS L7414-7468 `uniqueMods`, L7351-7396, L14655-14684. Strength changes re-apply them (JS L14486-14506).
- Not saved in the link (the element choice resets to the default on reload).

### 4.6 Other weapon-driven automatic behaviour
- **Faction-dependent element mods** (`add_weakness_dmg`, e.g. the "bane"-style mods that add the enemy's weakness): `addWeaknessDmgType` rewrites the mod's action to the element matching the current enemy faction (Infested -> Heat, Corpus -> Magnetic, Grineer -> Corrosive, ...; fallback Grineer). JS L7469-7497.
- **Impact -> Puncture swap** when any mod has `impactToPuncture`. JS L9653-9654.
- **Hidden auto-options** per attack: `isCoMult` forces the "damage per status is multiplicative" behaviour and `isWPCritMult` forces "weak point crit chance multiplicative" for that attack (changelog 2025.02.14 / 2025.02.19, index.txt L299-303). JS L9235, L9393, L8206-8208.

## 5. Mod system

### 5.1 Slot map (`modsPanel`, 30 entries; index = slot id; HTML L132-206)

| Slot ids | Slot | Shown for | Accepted mod `type` (search filter, JS L7126-7240 `initMods`) |
|---|---|---|---|
| 0-7 | 8 weapon mod slots (two rows of 4) | all weapons | `type` undefined or `exilus`; compatible by `categories`/`compTags` vs mod `tags`; not excluded by `uncomptag`; `trigger:"Semi"` mods only for Semi weapons; not already added; not mutually exclusive (`uncomp`) |
| 8 | Exilus | all weapons (forced visible for non-melee, JS L7800; never hidden for melee) | `exilus` |
| 9 | Stance | melee only (`.stancePanel` shown when `productCategory == "Melee"`) | `stance` whose tags hit the weapon's `compTags` (e.g. `NIKANAS_STANCE`) |
| 10 | Weapon arcane | all non-archgun weapons (melee, primary, secondary, exalted) | `weapon_mist` matching categories |
| 28, 29 | Two arcanes for archguns | `productCategory == "Archguns"` only | 28: `weapon_mist` tagged `primary`; 29: `weapon_mist` tagged `secondary` or the weapon's categories |
| 13-16 | 4 Warframe mod slots | all (inside the collapsible "Set warframe mods, aura, shards, warframe arcanes") | `frame_mod` matching categories or compTags |
| 17 | Aura | all | `aura_mod` |
| 11, 12 | 2 Warframe arcanes | all | `frame_mist` |
| 23-27 | 5 Archon shards | all | `shard` (18 entries: crimson/azure/amber/violet/emerald and their tauforged x1.5 versions, e.g. "+25% Critical Damage" / "+37.5% Critical Damage", "Incr. max stacks of Corrosion +2/+3", "Toxin Status +30%/+45% more damage", "+30%/+45% Primary Electricity Damage", "+10%/+15% Ability Damage affected by Electricity") |
| 18-22 | Incarnon evolution tiers | Incarnon weapons | weapon-defined perks (section 4.1) |

Mod-type counts in the bundle: 12 aura, 36 exilus, 17 frame arcanes, 27 frame mods, 18 shards, 65 stances, 42 weapon arcanes (`grep 'type: "..."'` over JS L2-5435). Mod sets (`set`) give set bonuses scaled by how many set pieces are installed (`getCountSetMods`, JS L8514-8518, used in L8705-8707).

The "Set warframe mods..." button toggles visibility of slots 11-17 and 23-27 (JS L14855-14857). Changelog 2026.03.21: frame/aura mods are also allowed on archguns.

### 5.2 Mod search popup
- Clicking any slot (after a weapon is chosen; otherwise alert "Need select a weapon first") opens `.modsSearchPopup` at the slot, focused on `#modsSearchInput` ("Mods/stats name"). Search matches the localized mod name **or any localized stat key in its action** (including `element`, `WITH_COND`, `phys` sub-keys), i.e. you can search by stat. Each result shows name + card image (fetched from `/getImg.php?imageName=..&locale=..`), a `title` tooltip with the mod's `info`, and a large card preview on hover. JS L7008-7026, L7126-7265, L14058-14108.
- Duplicate guard: `isModAlreadyAdded` hides a candidate if any installed mod's name *contains* the candidate's name (so "Heated Charge" is hidden while "Primed Heated Charge" is installed, not the reverse). JS L6998-7007. Mutual exclusion via `uncomp` lists (JS L6987-6997).
- Arcane (10), frame arcane (11-12) and shard (23-27) tiles show the arcane name under the tile and a `title` tooltip with the arcane/shard text (changelog 2023.12.19, 2025.04.14). JS L7518-7560.

### 5.3 Slot interactions
- Remove a mod: right mouse button on the tile, the small X (`.modInsideMRemove`), Delete key, or drag it and drop outside any tile. JS L8387-8429 `removeMod`, L14130-14173, L14264-14272.
- Drag a mod onto another of slots 0-7: the two swap (order is irrelevant to damage except element combination order). JS L14130-14168.
- "Trigger all conditions for mods with conditions" (section 8) decides whether `WITH_COND` values replace the unconditional ones.

## 6. Riven builder

- Button "Create riven mod" (`.createRiven`) opens popup `#rivenSettings` with a live card preview (title = weapon name, 4 preview lines) and **4 property rows**. Each row: a stat dropdown (`.rivenAttr`, `data-link` 1-4) and a value input `#rProp1..4` (`type=number`, step 0.1, min -9999, max 9999, placeholder 0; the change handler clamps to the HTML min/max and falls back to ±500 only if the attributes are missing; `e`/`+` keys blocked, `,` becomes `.0`). Reset (x) per row. HTML L437-510; JS L14685-14845.
- **Stat list** (built when a dropdown opens, JS L14701-14745). Key -> label (`addt` tag):
  - all weapons: `crit_chance` Critical Chance, `crit_mult` Critical Multiplier, `status_chance` Status Chance, `base` Damage, `speed` Fire Rate / Attack Speed, `Cold`/`Heat`/`Electricity`/`Toxin` (element), `Impact`/`Puncture`/`Slash` (phys), `addViral` Viral, `addBlast` Blast, `addGas` Gas, `addMagnetic` Magnetic, `addRadiation` Radiation, `addCorrosive` Corrosive, `status_damage` Status Damage, `Corpus`/`Grineer`/`Infested`/`Scaldra`/`Techrot` Damage to <faction> (SMITE), `crit_chance_weakp` Weak Point Critical Chance (SMITE), `mult_for_head` Weak Point Damage, `zoom` Zoom, `recoil` Weapon Recoil, `status_duration` Status Duration;
  - guns only: `multishot`, `ammoCapacity` Ammo Maximum, `ammoEff` Ammo Efficiency, `magazineSize` Magazine Capacity, `reloadTime` Reload Speed, `shot_speed` Shot Speed, `punch_through` Punch Through;
  - melee only: `comboDuration` Combo Duration, `initialCombo` Initial Combo, `melee_combo_eff` Heavy Attack Efficiency, `range` Range.
  - Each stat can be chosen once; at most 3 of the 4 base elements can be selected (`3 > J`).
- **Value units** (JS L7280-7293 `showRiven`): `comboDuration`, `initialCombo`, `range`, `punch_through` are absolute; faction stats are entered as a **multiplier** (e.g. 1.45 or 0.55) and stored as `v - 1`; everything else is a percent and stored as `v / 100`.
- **Special cases**: `speed` also sets `charge_time`, and for `primary-bow` weapons both are doubled (`FOR_WEAPON["primary-bow"]`); a Grineer riven stat also applies to "Kuva Grineer" and Infested to "Infested Deimos" (changelog 2026.01.09). JS L7298-7325.
- **Disposition: not modelled at all.** No riven disposition, no roll ranges, no positive/negative count rules; values are free numbers (no `dispos`/`omega` anywhere in the bundle).
- "Add riven" validates (error "Riven has null values." if a selected stat has 0, a value has no stat, or nothing is selected), then builds pseudo-mod `modsProps[9999]` named "Riven" (tags primary, secondary, melee, primary-archgun; no type, so it goes in slots 0-7) and tells the user to search "riven" in a slot; re-adding updates an installed riven in place. JS L14872-14905, L7273-7341.

## 7. Enemy selector and enemy panel

- **Enemy search** `#enemies` ("Type or select enemy name") filters `/data/json/enemies-min.json` (498 entries in samples/enemies-min.json) by localized name **or faction**. Selecting one resets Steel Path and Eximus to off. JS L6618-6658, L14209-14217.
- **Enemy panel** (`showEnemy`, JS L7687-7779): name, faction, **Enemy lvl** input `#setEnemyLvl` (min = enemy base level `b_lvl` or 1, max 9999; empty -> base level), **Damage reduction** % from armor, **Innate dmg reduction** per layer (`innateDR`), **Status immunity** list (`immun.status`), **The Steel Path** checkbox `#eSP`, **Eximus** checkbox `#eEximus` (hidden when the enemy has `eximusOff`), **Number of players in a group** `#eParty` (1-4, default 1; only for enemies with `showPartyCount`; health x {1:1, 2:1.5, 3:2, 4:3}, JS L5446, L8041-8042), a unique-damage-reduction warning with a wiki link (enemies with `unique`), the faction weakness/resistance list (`typeOfFaction`, JS L5503-5518), and scaled Health / Shield / Armor / Overguard values.
- **Level scaling** (`correctEnemyLvl`, JS L8039-8137): per-faction health/shield/armor curves with a smoothstep blend between level-offset 70 and 80 (`trans`, JS L7611-7616); Eximus uses separate multipliers (`enemyEximusArmor/Health/Shields`, JS L7637-7686); armor is clamped to [200, 2700] after scaling (with the strip applied to the clamp values); overguard from `getEnemyOverguard` (JS L7624-7636; base 12 for Eximus without own overguard).
- **Steel Path** multiplies base health and shield by 2.5 (JS L7700-7704). It does **not** add +100 levels; the level box is the final level. Wiki: Steel Path = +100 levels, 250% health and shield (The_Steel_Path.wiki L10-12, L59).
- **Damage reduction display** = `sqrt(3 * armor) / 100` (JS L8148-8150). This equals the wiki's enemy formula `0.9 * sqrt(armor / 2700)` for armor <= 2700 (Armor.wiki L33-39).
- **Armor strip**: "External Decrease Armor" option (section 8) is applied here, not in the hit formula.
- Enemy data keys seen: `name, faction, shield, health, armor` (all), `unique` 114, `eximusOff` 110, `immun` 52, `b_lvl` 36, `innateDR` 32, `elemRes` 22, `showPartyCount` 18, `maxProcStacks` 14, `overguard` 4.

## 8. "Select additional options" panel

Header text: "Any change starts a new random queue." Every checkbox / number change runs `setAdditionalParams` then `reCalc` (JS L12029-12114, L14507-14513). Number inputs are validated by `optCheckErrors` (non-numeric or <= 0 -> min; > max -> max; JS L12021-12028).

| Option (HTML id) | Inputs, range, default | What it changes (code) | Link key |
|---|---|---|---|
| External Virus (`optVirus`, `optVirusStacks`) | checkbox off; stacks 1-10, default 10 | Adds N Viral stacks to the enemy for the whole queue (JS L10782-10793) and counts as one extra status type for per-status mods if the weapon has no viral (JS L12015-12020) | `o.v {is_set, stucks}` |
| Strength (`strengthInput`) | number 0-1000, **displays 0**, placeholder 100; internal default 1.0 | `strengthMult = value/100`. Scales exalted weapons with `strengthType` simple/excalibur/mesa, ability-strength element mods (`add*S`), Chromatic Blade / Reactive Storm status chance, `flat_change_dmg_by_strength` (JS L14486-14506, L8466, L8586-8626, L9640-9643, L10372-10376) | `o.t` (multiplier) |
| Headshot/Weak Point bonus active (`optHeadshot`) | checkbox off | Applies headshot multiplier (`3 + mods + zoom + unique`), weak-point damage mods, weak-point crit chance mods; disabled per attack by `no_headshot_mult` (JS L9070-9084, L10355-10366, L8195-8210) | `o.hs` |
| External Decrease Armor (`optDecreaseArmor`, `...Val`) | checkbox off; 0-99 %, default 0 | Reduces enemy armor by that % in `showEnemy`/`correctEnemyLvl` (JS L7698-7712, L8040, L8113-8115) | `o.d {is_set, val}` |
| Stealth bonus (for melee) (`optStealth`) | checkbox off | Melee only: replaces each hit's crit tier `c` by `7` if `c == 1`, else `8 + (c - 1)` (`getStealthDmgBonus`, JS L8330-8336, applied L10826) | `o.s` |
| Heavy attack (for melee) (`optHeavy`, start from `optHeavyStartCounter`) | checkbox off (highlighted for melee); start combo 1-13, default 12 | Simulates heavy attacks for attacks without an `isHeavy` flag; combo multiplier starts at the given value and decays as heavy attacks consume combo, scaled by Heavy Attack Efficiency (JS L12158-12181 `addTimelineMeleeCombo`, L13253-13258) | `o.h` (start value not saved) |
| Light attacks combo multiplier (for melee) (`optMeleeCombo`, `optMeleeComboVal`) | checkbox off; 1-13, default 12 (internal default combo counter 11 = x12) | Combo tier used by "per combo" crit/status mods for light attacks (JS L8219-8225, L9905-9911) | not saved |
| Combo Ability Multiplier (`optAbilityCombo`, `...Val`) | checkbox off; 1-12, default 12 | Exalted weapons only: multiplies damage by the ability combo value (JS L10379-10387) | `o.ac {is_set, val}` |
| Sniper combo multiplier (for sniper rifle) (`optSniperCombo`, `...Val`) | checkbox off; 1.5-100, default 1.5 | Attacks with `sniperCombo`: multiplier; 1.5 when the box is off (JS L10325-10329) | `o.sc {is_set, val}` |
| Roar (Rhino) (`optRhino`, `optRhinoPc`) | checkbox off; 1-9999 %, default 30 | Added to the faction multiplier: `faction = (F != 1) ? F + roar : 1 + roar` (`getFactionMod`, JS L9104-9109) | `o.r {is_set, val}` |
| Eclipse (Mirage) (`optMirage`, `optMiragePc`) | checkbox off; 1-9999 %, default 30 | Separate damage multiplier `1 + val/100` (JS L9156, L9249, L9493, L10427, L10440) | `o.z` |
| Xata's Whisper (Xaku) (`optXata`, `optXataPc`) | checkbox off; 1-9999 %, default 26 | Extra Void damage instance per hit = round(sum(hit damage) x armor/innate DR factor x val/100), x1.5 vs overguard (`getXataDmg`, JS L13226-13243) | `o.x` |
| Toxic Lash (Saryn) (`optTLash`, `optTLashPc`) | checkbox off; 1-9999 %, default 30 | Extra toxin direct hit plus a toxin DoT (7.5 s) per hit (`toxicLashDmg` JS L13215-13225; L10571, L11463-11496) | `o.l` |
| Nourish (Grendel) (`optNourish`, `optNourishPc`) | checkbox off; 1-9999 %, default 45 | Adds Viral damage = base x val/100 (same path as `addViral` mods, JS L13019-13026) | `o.n` |
| Flat crit. chance (~Kullervo, Harrow) (`optCrit`, `optCritPc`) | checkbox off; 1-9999 %, default 50 | Absolute crit chance added after mods (`getFinalWeaponCritChance`, JS L8195) | `o.j` |
| Manual selection of the total number of statuses on the enemy (`optCountStatusOnEnemy`, `...Pc`) | checkbox off; 1-50, default 1 | Overrides the status-type count used by per-status mods (Condition Overload style) (JS L12015-12020, L10757) | `o.ms {is_set, val}` |
| Property "damage per status" calculated as multiplicative (~Condition Overload) (`optMultiplicativeBasePerStatus`) | checkbox off | Treats `base_per_status` as a separate multiplier instead of adding to base-damage mods (JS L9235, L9393-9405, L10831-10847); forced per attack by `isCoMult` | `o.md` |
| Madurai (physical damage increased by 30%) (`optMadurai`) | checkbox off | Physical types x1.3 except `POWER_WEAPON` (exalted) (JS L9255, L9495) | `o.g {is_set}` |
| (hidden) Weak-point crit chance multiplicative (`optMultiplicativeCC`) | no control in the HTML; only set from a link or per attack via `isWPCritMult` | JS L8206-8210, L12106-12107 | `o.w` |
| Trigger all conditions for mods with conditions (`optAllModsConds`) | checkbox **on** | `isMaxCond`: use `WITH_COND` mod values, unique-weapon conditional values and the last zoom level's bonuses (JS L8668-8744, L8782-8789, L9095-9103) | `o.m` |
| Add custom stat value (button) | opens popup, see section 9 | | `p.*` |
| Combine attack effects (button) | opens popup, see section 10 | | not saved |

## 9. "Add custom stat values" popup

Opened by "Add custom stat value" (needs weapon and enemy; JS L14930-14935). Each input (HTML L1116-1215) has `data-type` = internal stat key and `data-d`: when `data-d=1` the typed percent is divided by 100 (JS L15224-15241). Values clamp to the input's min/max. "Reset all" zeroes everything (JS L15242-15250). Stored with `setCustomStat` and added to the same sums as mods.

| Label | Key | Range, default | `data-d` | Link key |
|---|---|---|---|---|
| Base Damage | `base` | 0-999, 0 | 1 | `p.a` |
| Base Damage per Status Effect | `base_per_status` | 0-999 | 1 | `p.b` |
| Increase Base Damage by +X | `flat_change_dmg` | 0-9999 | 0 | `p.z` |
| Combo Ability Multiplier | `abilityCombo` | 1-12, 1 | 0 | `p.u` |
| Critical Chance | `crit_chance` | 0-999 | 1 | `p.c` |
| Critical Chance (multiple with others) | `mult_crit_chance` | 0-999 | 1 | `p.d` |
| Critical Chance (absolute bonus after mods) | `flat_crit_chance` | 0-999 | 1 | `p.e` |
| Weak Point Increase Critical Chance | `crit_chance_weakp` | 0-999 | 1 | `p.f` |
| Critical Multiplier | `crit_mult` | 0-999 | 1 | `p.g` |
| Critical Multiplier (absolute bonus after mods) | `crit_mult_add` | 0-999 | 0 | `p.h` |
| Critical Multiplier (multiple after all sources) (1 + x) | `crit_mult_mult` | 0-999 | 0 | `p.i` |
| Status Chance | `status_chance` | 0-999 | 1 | `p.j` |
| Vulnerable to Status Chance | `status_chance_mult` | 0-999 | 1 | `p.k` |
| Status Chance (absolute bonus after mods) | `absolute_status_chance` | 0-999 | 0 | `p.l` |
| Status Damage (as "Elementalist" mods) | `status_damage` | 0-999 | 1 | `p.m` |
| Vulnerability Status Damage (as "Burning Hate" mod) | `vuln_status_damage` | 0-999 | 1 | `p.o` |
| Fire Rate / Attack Speed | `speed` | 0-999 | 1 | `p.p` |
| Multishot | `multishot` | 0-999 | 0 | `p.q` |
| Magazine Size Increase | `magazineSize` | 0-999 | 1 | `p.r` |
| Reload Time Increase | `reloadTime` | 0-999 | 1 | `p.s` |
| Headshot Increase Damage (as "Deadhead" Arcane) | `mult_for_head` | 0-999 | 1 | `p.t` |
| Weak Point Increase Damage (as "Primary Acuity" mod) | `headshot_mult` | 0-999 | 1 | `p.v` |
| Initial Combo | `initialCombo` | 0-999 | 0 | `p.w` |
| Damage Vulnerability | `dmgVulnerability` | 0-999 | 1 | `p.x` |
| Heat Inherit | `heatInherit` | 0-999 | 1 | `p.y` |
| Add Heat Damage Instance (like Ember augment for 1st ability) | `heatAdd` | 0-999 | 1 | `p.aa` |

Link keys from JS L9724-9750 (save) and L6835-6946 (restore). There is no `p.n`. Note that the labels of `mult_for_head` and `headshot_mult` look swapped relative to their in-code meaning (`headshot_mult` adds to the x3 headshot multiplier, `mult_for_head` is a separate final multiplier, JS L9070-9084); see quirks.

## 10. "Combine attack effects" popup

Lists the weapon's `comb` groups (e.g. Acceltra Prime `[[0,1]]` "Rocket Impact" and "Rocket Explosion"; Lenz `[[0,1,2]]`; Catabolyst `[[1,2],[3,4]]`), each with a checkbox; text: combines the status effects that modify damage (heat/corrosive armor strip, cold crit damage, viral/magnetic) between a direct hit and its side attack, without adding their DPS together. If the weapon has no groups: "This weapon does not have any attacks for combining." Checked groups are read in `createDmgQueue` (JS L12407-12413) and share one status timeline. HTML L1059-1068; JS L7992-8005, L14514-14516, L14936-14939. Not saved in the link.

## 11. Simulation controls

- **Queue length slider** `#queueLen` "Continuous firing / attack duration (sec)": range 1-60, step 1, default 20; live label. Sets `queueLength`, recalculates. HTML L389-393; JS L14863-14871. Not saved in the link.
- **Every recalculation draws a new random queue** (crit tiers, status rolls, multishot rounding are sampled), so numbers jitter between runs (help text index.txt L128-145).
- **"Calculate average values over several full firing/attack cycles"** (`.butNewQueue`): clears the accumulators, runs `reCalc` on a `setInterval(…, 0)` loop for **5 s wall-clock**, then shows per attack and per layer the **median** of all runs, with a tooltip block "Min / Avg / Max" (the "Avg" cell is also the median), plus the median TTK, median "median per bullet" and median "avg per bullet (total)". The number of runs depends on CPU speed. Requires weapon and enemy. JS L14273-14485.

## 12. Result panels

Container `.resultWeaponArmor` with four panels: **Armor**, **Health**, **Shield**, **Overguard** (HTML L394-420). Health always; Armor if the enemy has armor; Shield if it has shields; Overguard if `currOverguard > 0` (JS L9140-9153 `reCalcDisplay`). Each panel has an **Expand / Collapse** button that enlarges one panel and shrinks the others (JS L14035-14057). Hovering the panel title shows an explanation (JS L15117-15193):
- Health: "average damage to the Health layer (or against an enemy with 100% armor stripped). TTK ... considering Overguard, Shield, Health. Armor damage reduction is not applied."
- Armor: "average damage to the Armor layer. TTK ... Health (with armor damage reduction), Overguard, Shield."
- Shield: "... Toxin damage is not applied to Shield and bypasses it directly to the Health layer."
- Overguard: "average damage to the Overguard layer."

Per attack row (`createDamageInfoHTML`, JS L12817-12844):
| Output | Meaning | Code |
|---|---|---|
| Attack name (hover: modded per-type damage of that attack, copied from the weapon panel) | | JS L15124-15142 |
| **Random queue dps** | Median over the 1-second buckets of the queue (`getDmgPerSecond` + `median`) | JS L12787-12805, L12590-12596 |
| avg per bullet (without status dmg) | mean over hits of direct damage + Xata + `dmgOnStatusEff` + Toxic Lash direct | JS L12217-12232 `getAvgQueueVal` |
| avg per bullet (status dmg) | mean over hits of the full-duration damage of the statuses the hit applied (tick x duration x count) | JS L12197-12216 |
| avg per bullet (total) | sum of the two above | same |
| median per bullet (total) | median of per-hit `totalDmg` (changelog 2025.06.09) | JS L12233-12238 |
| TTK | seconds (3 decimals) or "not killed" within the queue; Health/Shield/Overguard rows share one layered TTK (overguard -> shield -> health), Armor row uses the armored-health arrays | JS L12681-12697 `calcTTKWithStatusUnified` (L13593-13804) |
| "More details" | opens the extended queue popup (section 13) | JS L12840-12842 |

Internally the queue is simulated once per layer (`armor`, `health`, `shield`, `overguard`, plus `healthFull`/`armorFull` when the enemy has shields, used for TTK) (JS L12555-12680).

There is **no status-uptime chart, no DPS-over-time chart and no per-element DPS table**. Status state is only visible per hit in section 13.

## 13. Extended damage queue popup ("More details")

Popup `#dmgInfo` (HTML L1069-1110, JS L14957-15042, renderer `g()` L13827-13957): one collapsible block per (layer x attack), the clicked one first and open. Each block is a horizontal timeline (drag to scroll, mouse wheel scrolls horizontally, JS L13982-14021, L15110-15116), rendered in chunks with `requestIdleCallback`. Shot cards and reload cards are merged and sorted by time.

Shot card fields: hit number and time (s); hit damage; crit multiplier (only if > 1; card highlighted); armor reduction % active on that hit; melee: stance multiplier and heavy-combo multiplier; Xata's Whisper damage; Toxic Lash direct hit and tick; `dmgOnStatusEff` damage (e.g. Cascadia Empowered); icons of statuses applied by the hit; 1-tick damage of each status on the hit; Primary Debilitate status tick damage (and a cold icon); **current stack count of every status on the enemy**. Reload card: "↻ #n" with time. A legend explains each field (HTML L1075-1108).

## 14. Tooltips (summary)

Mod list `title` = mod `info`; arcane and shard tiles `title` = info; attack-name hover = modded per-type damage; result-panel title hover = layer explanation; Min/Avg/Max block after "average cycles"; riven card preview. JS references in sections 5, 11, 12.

## 15. Build link (URL) encoding

- On every `reCalc` (except during "average cycles") the build is serialised to JSON, compressed with `LZString.compressToEncodedURIComponent`, and written with `history.pushState` as `?p=<data>&n=1`. Reading: `n` present -> LZString; absent -> legacy `atob` base64 JSON. JS L9663-9786 (`reCalc` tail, `setLocation`, `getSettingsFromUrl`); LZString L13274-13592.
- Restore (`restoreSettings`, JS L6677-6954) runs once the enemy list is loaded (polled every 5 ms, L6955-6958): loads the weapon, then enemy, level, SP, Eximus, progenitor, zaw/kitgun parts (by triggering dropdown clicks), riven rows, mods (`showMod`/`showIncMod`), options, then custom stats (deferred), then `reCalc`.

### Key map, decoded with research/test-cases/user-example-furis.json

| Key | Meaning | Furis example |
|---|---|---|
| `w` | weapon name (`currWeapon.name`) | "Furis" |
| `e` | enemy name | "Corrupted Heavy Gunner" |
| `l` | enemy level, string from `#setEnemyLvl` | "1000" |
| `sp` | Steel Path 1/0 | 1 (on: health and shield x2.5) |
| `ex` | Eximus 1/0 | 0 |
| `j`, `jv` | progenitor element, fraction | absent (not a Kuva/Tenet/Coda weapon) |
| `z.{g,l}` / `k.{g,l}` | zaw grip/link, kitgun grip/loader | absent |
| `m.<slot>` | `0` = empty, mod name string, or for slots 18-22 `{mId, pId}` (Incarnon tier and perk index); `"Riven"` = the riven from `r` | 0 Primed Heated Charge, 1 Galvanized Diffusion, 2 Pistol Elementalist, 3 Creeping Bullseye, 4 Hornet Strike, 5 Primed Target Cracker, 6 Lethal Torrent, 7 Riven; 8 exilus, 9 stance, 10 arcane, 11-12 frame arcanes, 13-16 frame mods, 17 aura, 18-22 Incarnon (all empty, so no evolutions), 23-27 shards, 28-29 archgun arcanes all 0 |
| `o.hs` | headshot option | false |
| `o.s` | stealth | false |
| `o.m` | trigger all conditions | true |
| `o.h` | heavy attack | false |
| `o.md` | per-status multiplicative | false |
| `o.w` | hidden weak-point CC multiplicative | false |
| `o.v` | external virus `{is_set, stucks}` | off, 0 |
| `o.r` / `o.z` / `o.x` / `o.l` / `o.n` | Roar / Eclipse / Xata / Toxic Lash / Nourish `{is_set, val %}` | all off, 0 |
| `o.j` | flat crit chance `{is_set, val %}` | off |
| `o.d` | external armor strip `{is_set, val %}` | off |
| `o.ms` | manual status count `{is_set, val}` | off |
| `o.g` | Madurai `{is_set}` | off |
| `o.t` | ability strength multiplier (1 = 100%) | 1 |
| `o.sc` | sniper combo `{is_set, val}` | off, 1.5 |
| `o.ac` | ability combo `{is_set, val}` | off, 12 |
| `p.*` | custom stats in internal units (percent fields already /100), see section 9 | all 0 |
| `r.<1-4>` | riven rows: `t` English label, `p` stat key, `a` addt tag (`""`, `element`, `phys`, `SMITE`), `v` typed value (string) | 1: Critical Multiplier `crit_mult` +111.4 %; 2: Multishot +148.1 %; 3: Status Damage `status_damage` +111.4 %; 4: Zoom `zoom` -79.3 % (zoom has no damage effect) |

Default values when the option was never touched: `o.v.stucks` stays 0 and buff `val`s stay 0 even though the inputs show 10/30/... (`setAdditionalParams` only copies input values on a change, JS L12029-12114), which is why the Furis example has zeros.

Not stored in the link: queue length, heavy start counter, light-combo option, stance combo choice, combine-attack checkboxes, Chromatic Blade / Reactive Storm element, party size.

## 16. Localization

- One page per language (`/`, `/zh/`, `/ru/`, `/ja/`, `/ko/`, `/es/`, `/fr/`); server-rendered static text plus a `localeLang` dictionary injected per page (`var localeLang = {}` for English, HTML L52) and `lang` (HTML L47).
- `gL(key)` looks up `localeLang[key]`, then lowercase, then capitalised, else returns the key (JS L7405-7413). Used for every dynamic string, mod/weapon/enemy/faction names (`localeName`, `lFactionName`), and searches use localized names (JS L6659-6676, L6978-6981, L7027-7031). The optimizer uses `gL` too with `{1}`-style placeholders (JS L15297-15310).
- Changelog note: Spanish is "interface only" (index.txt L369).
- Project decision: English only, but through an i18n layer (CLAUDE.md).

## 17. Optimizer ("Autobuild" / "Build optimizer")

A button **Autobuild** is appended under the mod block; it opens modal `#wfoptModal` "Build optimizer" (JS L21471-21719 `Ya`, open `Xa` L21410-21460). The optimizer drives the same global state and `reCalc` engine as the page.

### 17.1 Controls (left column)
| Group | Control (id) | Range / default | Notes |
|---|---|---|---|
| Metric (radio `wfoptMetric`) | DPS (`dps`, default), Minimum TTK (`ttk`), Maximum hit damage against a protection layer (`hit_infight`) | | A 4th metric `hit_fresh` exists in code (`fa`, JS L16019-16051; `me` L21863-21868) but has no radio |
| Calculation target | Attack (`wfoptAttack`) | the weapon's attacks, default first | changing it updates heavy/incarnon/search window |
| | Defense layer for ranking (`wfoptDefense`) | `auto` (= armor if the enemy has armor, else health) or health / armor / shield / overguard, each labelled with the layer's value (`lc`, JS L21031-21041) | |
| | Heavy attack (`wfoptHeavy`) + combo counter (`wfoptComboVal`) | off; 1-13, default 12 (copied from main `optHeavyStartCounter`) | disabled and forced when the attack has `isHeavy` (JS L21333-21346) |
| | Search window, s (`wfoptQueueLen`) | 1-60; on open set to `round(100 / attack speed)` clamped 5-20 (10 if speed unknown) (JS L15422-15429, L21328-21332) | warns if > 250 shots per evaluation or < 5 s (JS L15430-15455) |
| | Beam width (`wfoptBeamW`) | 1-8, default 3 | beam-search width |
| | Exclude elements (`wfoptElemExclude`) | free text, e.g. "gas, blast" (split on `,` `;` space, localized names accepted, JS L17773-17797) | |
| Build rules | Use weak point damage (`wfoptHeadshot`) | off | off also auto-excludes weak-point-only mods |
| | No faction bonus (drop SMITE mods) (`wfoptNoFaction`) | **on** | |
| | No on-kill bonuses (`wfoptNoOnKill`) | off | drops `WITH_COND` of `onkill` mods / excludes kill-only mods |
| | Mod conditions are met (`wfoptMaxCond`) | on | sets `isMaxCond` |
| | Keep installed mods (`wfoptKeepExisting`) | off | locks occupied slots (JS L16303-16322 `Dc`) |
| | Keep progenitor element (`wfoptLockProg`) | off; shown only for Kuva/Tenet/Coda; refuses if none selected | otherwise phase 3 also picks the progenitor element and % |
| | Use incarnon perks (`wfoptIncPerks`) | shown and checked only for Incarnon weapons | searches Incarnon evolutions |
| Warframe slots | Warframe mod (1 slot only) (`wfoptFrameMods`), Warframe arcanes, Aura, Archon shards | all off | off = those slots are cleared before the search; warframe mods only in slot 13 (JS L16323-16344 `dc`) |
| Warframe ability strength | Ability strength, % (`wfoptStrength`) | 1-2000, default = main strength (100) | |
| Allowed external buffs | per buff checkbox + value: Toxic Lash (30), Xata's Whisper (50), Roar (100), Eclipse (100), Nourish (50), External viral (stacks) (10, max 10), Absolute critical chance (50) | enabled only if the main page has the matching global; initialised from main-page state | list `ib`, JS L21731-21795; phase 7 decides which allowed buffs to keep |

Defaults as an API object: `WFOpt.defaultCtx` (JS L21938-21975).

### 17.2 Right column
- **Build it** (`wfoptStart`; becomes **Abort** while running), **Clear log**.
- **Prohibited mods for build: N** (collapsible): search box ("mod name or stat", up to 40 results, compatible non-shard mods only), click to add, chips with x to remove; persisted in `localStorage["wfoptExclNames"]`; first-run defaults with reasons: Mecha Empowered, Proton Snap, Arcane Blade Charger, Arcane Primary Charger, Arcane Arachne (JS L16424-16507, L21355-21409, `ae` L21848-21856).
- **Diagnostics by phase** (hidden with class `dNone`): Estimate check, 1 profile, 2 sensitivity, 3 elements, 4 assembly, 5 polish, 6 converge, 7 buffs.
- Progress bar with text; log pane (monospace, max 500 lines, colour classes head/ok/warn/err/dim) (JS L15314-15343).

### 17.3 Workflow (`Nd` runFull, JS L20832-21021)
1. Snapshot the page state (`ja`, JS L15463-15503); apply context (`ec`, JS L16585-16607); remove installed mods that fall under exclusions (`vc`, JS L16272-16302) and disallowed warframe slots (`dc`).
2. Phase 3 elements and progenitor element; Phase 3.5 warframe slots / aura / shards / perks prefill (skipped for TTK, with an explanation in the log); Phase 4 portfolio of starts + beam search (needs >= 2 free main slots); estimate-trust check; Phase 6 converge to a fixed point; Phase 7 external-buff portfolio (only if any buff allowed); "Shake" (remove the weakest three and rebuild, 2 rounds); honest final choice between candidates (`Md`); check (`yd`).
3. Stance is chosen by trying every compatible stance and keeping the best (first combo of the stance is selected; the log tells the user to pick another by hand) (JS L15608-15653).
4. Automatic exclusions per candidate (`jd`, JS L16203-16271): mods that only extend status duration; SMITE mods when "No faction bonus"; kill-only mods when "No on-kill"; weak-point-only mods when headshot is off; mods whose `info` matches "next shot / first shot after" or "on ability cast" unless the metric allows it. Stats ignored as damage-neutral listed in `Sc`/`Ic` (JS L21796-21813, L21876-21890).
5. During the search the RNG is replaced by a deterministic low-discrepancy sequence (golden-ratio stride for `Math.random`, hashed accumulator for `isProc`) so candidate comparisons use common random numbers (`sa`/`Ca`/`ta`, JS L15688-15718). An evaluation cache is reported at the end.
6. Mod data field `prefer` (±25 %) biases choices; the log reports when it changed a pick (JS L16355-16404).
7. On success the best build is written to the page (`ad`, JS L18659-18705), options are synced back (`ob`, JS L16614-16637), `reCalc` runs, and the log ends with "=== DONE in X s ===", before/after scores and the slot list, plus "Build ready. Return to the main window to see it." On abort or error the original snapshot is restored (`sb`, JS L16642-16656).
8. TTK scores are displayed as "TTK x s", "TTK 0 s (margin xN)" for one-shots, or "not killed, EHP/DPS ~N s" (JS L15369-15386).

Public API on `window.WFOpt` (JS L21932-22036): `open, close, runFull, runValidate, runProfile, runSensitivity, runElements, runBeam, runPolish, runConverge, runBuffPortfolio, applyLastBuild, getLastBuild, stop, defaultCtx`, plus internals.

## 18. Building guide page

`/en/guide/` (guide.txt): a static article of build advice (AoE vs direct, per-status mods, multishot, elements by faction, fire rate, crit, forced procs, beams, headshots, punch through, ammo efficiency, unique enemies, exalted strength, riven choice) plus the same help and changelog. No interactive features. Useful as user-facing documentation, not a calculator feature.

---

## Suspected inspiration quirks

1. Strength input shows `0` on load while the engine uses 100 % (`strengthInput value="0"`, `strengthMult = 1`); typing 0 really sets 0 % strength (HTML L226; JS L5490, L14486-14489, L12021-12028).
2. External Decrease Armor is only applied to the enemy's armor value in `showEnemy`/`correctEnemyLvl`; on Steel Path `showEnemy` skips the strip, so it only takes effect through the 200/2700 clamps there (JS L7700-7712 vs L8113-8115). Looks like a bug.
3. "Calculate average values over several full firing/attack cycles" runs for a fixed 5 s wall-clock, so the sample count depends on CPU speed, and the cell labelled "Avg" shows the median (JS L14286-14335).
4. `getDmgPerSecond` spreads each hit's `totalDmg` over the 1-second buckets by hit index (`p * queueLength / hits`), not by the hit's timestamp, so reload gaps are smeared out and the "median per second" is close to a mean (JS L12787-12801). Needs confirmation by the mechanics reviewer.
5. Combo counters accept up to 13 (`optHeavyStartCounter`, `optMeleeComboVal`, optimizer `wfoptComboVal` max 13) although the in-game melee combo multiplier caps at 12x (UNVERIFIED against the wiki in this file).
6. Ability combo: `Math.max(abilityComboСounterSettings.val, customStat abilityCombo)` uses the option's stored value (default 12) even when its checkbox is off, as soon as the custom stat is > 1 (JS L10379-10387).
7. `checkAlreadyAdded` always returns `""` (its `return` is inside an `.each` callback), so `showMod` never blocks duplicates; duplicates are only hidden in the search list (JS L7267-7272, L7501).
8. `isModAlreadyAdded` uses substring matching on names (`name.includes`), which can hide unrelated mods whose name is contained in an installed mod's name (JS L6998-7007).
9. Riven: no disposition or roll ranges at all; values up to ±9999 accepted (HTML L463-499). Unit display list contains a typo `"Infected"` so Infested faction rows show "%" (JS L14786-14796), and the restore path uses a different unit list without factions (JS L6741-6754).
10. Status durations are truncated to whole seconds: `curr_duration = parseInt(duration * (1 + status_duration))` (JS L9085-9093).
11. Zoom bonuses: only the last (highest) zoom level is used, and only when "Trigger all conditions" is on; there is no zoom selector (JS L9095-9103).
12. `setChromaticBlade` / `setReactiveStorm` write `currWeaponInitialAfterElementChange[0].damage` (the variable holds the attacks array) on apply but `.attacks` on reset (JS L7351-7396, L6974); harmless only by accident.
13. Several option values are not part of the build link (queue length, heavy start counter, light-combo, stance combo, combine-attack groups, exalted element choices, party size), so a shared link can reproduce different numbers.
14. After an optimizer run the global `queueLength` stays at the optimizer's search window (set in `ec`), while the page slider still shows the old value (`ob` does not sync `#queueLen`) (JS L16602-16603, L16614-16637). Needs a runtime check.
15. Dead controls: `wfoptAllowOneShot` is read but never rendered (JS L16566-16568, L21406-21409); `weaponFilter` global is unused although the changelog mentions a weapon filter (JS L5492).
16. Custom-stat labels for `headshot_mult` ("Weak Point Increase Damage (as Primary Acuity)") and `mult_for_head` ("Headshot Increase Damage (as Deadhead)") may be swapped relative to how the engine uses them (JS L9070-9084). Needs the mechanics reviewer to check which mod maps to which key.
17. The Health panel is "100 % armor stripped" damage, not damage to an armorless enemy of that type; users can misread it as the real DPS against an armored target (tooltip JS L15157-15168).
18. The help text lists deliberate simplifications the inspiration admits to: heat armor strip applied as 50 % from the first tick, Galvanized Savvy-like mods ignored on AoE, headshot multiplier disabled on AoE, attacks simulated independently, syndicate radial effects disabled, Nightwatch Napalm ignores unique DR (index.txt L147-195).

## Open questions

1. Whether the Incarnon Form attack's DPS is meaningful without the tier-18 "Incarnon Form" tile selected (the code computes all attacks regardless; not verified by running the page).
2. Exact behaviour of `hit_infight` vs `hit_fresh` metrics and of `Md` / `Pb` / `yd` final-choice steps; not traced here (belongs to an optimizer-algorithm study).
3. Whether the optimizer can pick the custom Riven (`modsProps[9999]` lies inside `modsProps.length` and has no `type`, so `Db`/`Nc` would accept it for slots 0-7) - not verified.
4. What `showPartyCount` enemies are exactly (changelog says Demolishers) and whether the party multiplier also scales shields (code: health only, JS L8100-8106).
5. Whether the stealth formula (`7`, or `8 + (tier - 1)`) is intended as "+700 % stealth damage replaces the crit tier" - semantics left to the mechanics research.

## Parity checklist

| Feature | Must-have for parity | Notes |
|---|---|---|
| Weapon search (localized substring, keyboard nav) | yes | 666 weapons; data from open sources, not the bundle |
| All attacks of a weapon simulated and listed separately | yes | no single attack picker in main UI |
| Melee stance slot + stance combo selector | yes | default stance tables when none |
| Heavy / slam attacks with `isHeavy` auto-mode | yes | |
| Weapon info panel with live modded stats and per-type damage | yes | |
| Incarnon evolutions (tiers 18-22) and Incarnon-form attack | yes | incarnon magazine, no ammo efficiency |
| Kitgun grip + loader | yes | |
| Zaw grip + link | yes | |
| Progenitor element + 25-60 % | yes | |
| Chromatic Blade / Reactive Storm element choice | yes | save it in the link (inspiration does not) |
| Faction-weakness element mods (`add_weakness_dmg`) | yes | |
| 8 mod slots + exilus + arcane (+2 archgun arcanes) | yes | |
| 4 warframe mods, aura, 2 warframe arcanes, 5 archon shards (incl. tauforged) | yes | |
| Mod search by name or stat, compatibility, exclusivity, Semi filter, set bonuses | yes | |
| Mod tile interactions (remove, swap by drag) | yes | keyboard equivalents too |
| Riven builder (4 stats, stat list per class, units, bow speed x2, faction multiplier) | yes | add disposition/ranges as an improvement (no) |
| Enemy search by name or faction | yes | |
| Enemy level, Steel Path, Eximus, party size, innate DR, immunity, unique-DR warning | yes | fix SP armor-strip bug |
| Enemy scaling (health/shield/armor/overguard, Eximus, 200/2700 armor clamp) | yes | cross-check with wiki |
| Every "additional option" in section 8 (19 controls incl. hidden `o.w`) | yes | |
| Custom stat values popup (26 stats) | yes | |
| Combine attack effects | yes | |
| Trigger all conditions toggle | yes | default on |
| Queue length 1-60 s | yes | |
| Random queue simulation, re-rolled on every change | yes | our engine may also offer a deterministic expected-value mode (no) |
| Average over many cycles with min/median/max | yes | use a fixed run count instead of 5 s |
| Result panels per layer (armor, health, shield, overguard) with expand | yes | |
| Per attack: queue DPS, avg per bullet (direct/status/total), median per bullet, TTK | yes | |
| Extended queue timeline (per-hit cards, reloads, status stacks) | yes | |
| Tooltips (mod info, arcane/shard info, attack damage types, layer explanations) | yes | |
| Build link encode/decode (all keys in section 15) | yes | import inspiration links (`p`, `n=1` LZString, legacy base64) is a nice extra (no) |
| Copy build link button | yes | |
| Localization (7 languages) | no | English only via i18n layer per CLAUDE.md |
| Optimizer: metrics DPS / min TTK / max hit | yes | |
| Optimizer: attack, defense layer, heavy + combo, search window, beam width, element exclusion | yes | |
| Optimizer: build rules (headshot, no faction, no on-kill, conditions, keep installed, keep progenitor, incarnon perks) | yes | |
| Optimizer: warframe slot toggles, strength, allowed external buffs with values | yes | |
| Optimizer: prohibited-mods list with persistence and defaults | yes | persist locally |
| Optimizer: progress bar, log, abort, apply result to the page | yes | |
| Optimizer: per-phase diagnostic buttons | no | hidden in the inspiration |
| Help popup and changelog | no | our own docs instead |
| Building guide article | no | |
| TOP 20 monthly chart, donations, Telegram, cookie consent | no | out of scope (CLAUDE.md) |
