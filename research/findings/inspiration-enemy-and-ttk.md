Status: draft

# Inspiration: enemy model, defense layers and time-to-kill

Reverse-engineered from `research/external/inspiration/min.pretty.js` (warframe-damage.com bundle; changelog in `guide.txt` runs to 2026-11-20). All line numbers refer to `min.pretty.js` unless another file is named. "Δ" below always means `level − b_lvl` (enemy level minus the enemy's base level, `b_lvl` defaulting to 1). Wiki comparisons point to `research/findings/wiki-enemy-defenses-and-scaling.md` (the sibling wiki research, abbreviated **WEDS**) and to cached wiki pages in `research/external/cache/wiki/`.

Scope: enemy data and its level scaling, the four defense layers, the armor formula, faction modifiers, immunities and stack limits, unique damage reduction, and the TTK simulation. How hits, crits and procs are rolled is covered by `inspiration-queue-model.md` and `inspiration-status-and-procs.md`; here they are only described as far as TTK consumes them.

---

## 1. Enemy data model

### 1.1 Fields in `enemies-min.json`
498 entries. Every value is a **string** (`"val": "1000"`), coerced by JS arithmetic. Fields (counts from the sample):

| Field | Count | Meaning in code |
|---|---|---|
| `name`, `faction` | 498 | `faction` selects the faction-modifier table (`typeOfFaction`, 5503-5518) and the scaling curve (`correctEnemyLvl` switch, 8055-8095). Matched verbatim, so `" Infested"` with a leading space is not Infested. |
| `health.val`, `shield.val`, `armor.val` | 498 | base values at `b_lvl`. `0` means the layer does not exist. |
| `b_lvl` | 36 | base level; default 1 (8047, 7602, 7606, 7616, 7625). The level input cannot go below it (7695-7697, 14186-14191). |
| `overguard.val` | 4 | base overguard (Kuva Trokarian 2, Thrax Legatus 10, Thrax Centurion 15, Scaldra Dedicant 22). |
| `immun.status` | 52 | list of lower-case status names, or `["all"]`. Stored in `enemyImmunStatus` (7713). |
| `innateDR` | 32 | `{health, armor, shield, overguard?, status?}` fractions, stored in `enemyinnateDR` (7715-7720, default all 0). `status` is read separately as `currEnemy.innateDR.status` (10896). |
| `elemRes` | 22 | per-type fractional resistance, keys lower case, e.g. `{"cold":0.75,...}` (Corpus "crewmans-rangers"), `{"heat":0.8}` (Hyekka). Applied as `× (1 − elemRes[type])`. |
| `maxProcStacks` | 14 | stored in `maxCountProc` (7714); caps stacks per status type. 4 for Acolytes, Archons, Void Angel, Demolisher Voidrig, H-09 Efervon Tank; 10 for Zealoid Prelate. |
| `showPartyCount` | 18 | shows the squad-size input (7756-7764); health × `healthPartyMult`. |
| `eximusOff` | 110 | hides the Eximus checkbox (7745-7751). |
| `unique` | 114 | unique damage-reduction key. **Overwritten with `"notset"` on load** (7690), see section 8. |

### 1.2 Loading (`showEnemy`, 7687-7779)
```
currEnemy = deep copy of Enemies[name]
if currEnemy.unique exists: currEnemy.unique = "notset"            # 7690 (confirmed in min.js)
currEnemyLvl = max(currEnemyLvl, b_lvl)                              # 7695-7697
p = decreaseArmorOpt.is_set ? decreaseArmorOpt.val/100 : 0           # 7698 ("decrease armor" option, %)
if SteelPath:                                                        # 7699-7703
    health.val = 2.5 * floor(base health)   (if base != 0)
    shield.val = 2.5 * floor(base shield)   (if base != 0)
    armor.val  = base armor string, unchanged, p NOT applied
else:                                                                # 7704-7709
    health.val = floor(base health)
    armor.val  = floor(base armor) * (1 - p)
    shield.val = floor(base shield)
enemyImmunStatus = immun.status or []; maxCountProc = maxProcStacks or 0; enemyinnateDR = innateDR or zeros
then correctEnemyLvl(currEnemyLvl)
```
The level input accepts 1-9999 (7725, 14189-14191). Steel Path does **not** add +100 levels: the user types the final level, which matches the WEDS rule "the Steel Path level bonus is included in the displayed level" (WEDS §8.1).

---

## 2. Level scaling (`correctEnemyLvl`, 8039-8138)

### 2.1 Smoothstep
`trans(L, U, level, b_lvl)` (7611-7616):
```
x = level - b_lvl
x < L  -> 0;  x > U -> 1;  else t = (x-L)/(U-L); return 3t² - 2t³
```
Health, shield and armor all use L=70, U=80 on Δ (8096-8098). Arithmetic is JS float64. The wiki's float32 interpolation `f1 + (f2−f1)·s` (WEDS §7.1) is not modelled; the inspiration computes `f1·(1−s) + f2·s`.

### 2.2 Curves
Every curve has the form `1 + C·Δ^E`. Defaults are set first (8048-8054) and overridden per faction by the `switch` (8055-8095). `24·√5/5 = 10.7331`, `30·√5/5 = 13.4164`, `36·√5/5 = 16.0997`, `30.5·√5/5 = 13.6400`.

| `currEnemy.faction` | Health low (x) | Health high (E) | Shield low (w) | Shield high (B) | Armor low (m) | Armor high (F) |
|---|---|---|---|---|---|---|
| default (anything not listed: Narmer, Stalker, Scaldra, Anarchs, Corpus Amalgam, Zariman, Tenno, "", UNKNOWN, " Infested", " Sentient", ...) | 0.015Δ^2 | 10.7331Δ^0.5 | 0.02Δ^1.75 | 1.6Δ^0.75 | 0.005Δ^1.75 | 0.4Δ^0.75 |
| Infested, Infested Deimos (8056-8060) | 0.0225Δ^2.12 | 16.0997Δ^0.72 | default | default | default | default |
| Corpus (8061-8066) | 0.015Δ^2.12 | 13.4164Δ^0.55 | 0.02Δ^1.76 | 2Δ^0.76 | default | default |
| Grineer, Kuva Grineer (8067-8071) | 0.015Δ^2.12 | 10.7331Δ^0.72 | default | default | default | default |
| Orokin (8072-8077) | 0.015Δ^2.1 | 10.7331Δ^0.685 | 0.02Δ^1.75 | 2Δ^0.75 | default | default |
| Techrot (8078-8083) | 0.015Δ^2.1 | 13.6400Δ^0.72 | 0.02Δ^1.76 | 3.4106Δ^0.76 | default | default |
| Sentient, Neutral, Murmur (8084-8094) | 0.015Δ^2 | 10.7331Δ^0.5 | 0.02Δ^1.75 | 2Δ^0.75 | default | default |

Blending (8096-8098):
```
ShieldMult = w(1-s) + B·s ;  HealthMult = x(1-s) + E·s ;  ArmorMult = m(1-s) + F·s ;  s = trans(70,80,level,b_lvl)
```

### 2.3 Final values (8099-8125)
```
party   = healthPartyMult[#eParty] if the party input exists else 1     # {1:1, 2:1.5, 3:2, 4:3} (5445, 8041-8042)
shield  = Eximus ? round(shield.val * enemyEximusShields(level))
                 : round(round(shield.val * ShieldMult * 100) / 100)
health  = (Eximus ? round(health.val * enemyEximusHealth(level))
                  : round(round(health.val * HealthMult * 100) / 100)) * party
armor   = Eximus ? round(armor.val * enemyEximusArmor(level))
                 : round(round(armor.val * ArmorMult * 100) / 100)
if armor > 2700:        armor = 2700 - 2700*d                           # d = decreaseArmorOpt fraction
if 0 < armor <= 200:    armor = 200 - 200*d
currShield, currHealth, currArmor (= currEnemyArmorVal) = those values
```
- `shield.val`/`health.val` already include the Steel Path ×2.5 (section 1.2), so Steel Path multiplies the scaled value by 2.5.
- The party multiplier applies to health only, and only for enemies whose UI shows `#eParty` (18 with `showPartyCount`: Demolishers, Demolyst units, Demolisher Voidrig). WEDS §8.3: Demolishers +50/100/200% health for 2/3/4 players. Matches.
- Cap 2700 and floor 200 match U36 (WEDS §3.2). Changelog: "set minimal armor as 200" (guide.txt 2024-07-18), "Fixed error with decrease armor option when armor equal 2700" (2024-07-09).
- Rounding: armor uses `round`, the wiki gadget uses `floor` (WEDS §3.2). Different by at most 1 armor point.
- Overguard is set only if Eximus is checked or the enemy has base overguard (8126-8137). The "Damage reduction" text shows `100·sqrt(3·armor)/100` and ignores innate DR (8133, 8136).

### 2.4 Eximus (7637-7686)
Eximus ignores the faction curves. It uses the **default** health/shield curves and the armor curve, times a step factor `k` that depends on the **absolute level `b`**, not on Δ:

`enemyEximusHealth` (7653-7669) and `enemyEximusShields` (7670-7686) share one step factor:

| absolute level b | k_hs |
|---|---|
| b ≤ 15 | 1 |
| 15 < b ≤ 25 | 1 + 0.025(b−15) |
| 25 < b ≤ 35 | 1.25 + 0.125(b−25) |
| 35 < b ≤ 50 | 2.5 + (2/15)(b−35) |
| 50 < b ≤ 100 | 4.5 + 0.03(b−50) |
| b > 100 | 6 |

```
EximusHealthMult = k_hs * blend(1+0.015Δ^2, 1+10.7331Δ^0.5)       # b ≤ 50: low curve only; b > 100: high curve only
EximusShieldMult = k_hs * blend(1+0.02Δ^1.75, 1+2Δ^0.75)
```
For b ≤ 50 the code multiplies by the low curve only (no smoothstep). For 50 < b ≤ 100 it blends with `trans(70,80)`. For b > 100 it uses the high curve only (7666, 7683), even if Δ < 80 (possible when b_lvl > 20, e.g. Necramech b_lvl 40).

`enemyEximusArmor` (7637-7652):

| absolute level b | k_a |
|---|---|
| b ≤ 25 | 1 |
| 25 < b ≤ 35 | 1 + 0.0125(b−25) |
| 35 < b ≤ 50 | 1.125 + 0.06667(b−35) |
| 50 < b ≤ 100 | 1.375 + 0.005(b−50) |
| b > 100 | 1.625 |

`EximusArmorMult = k_a * ArmorCurve` (same piecewise structure as above). The 2700 cap still applies.

Comparison with WEDS §8.2 (wiki gadget): the wiki adds an Eximus bonus to **base** health `max(1.1·Base, c·(Base+900)·k(Δ))` with c = 0.25 or 0.375, keeps the **faction** curve, and gives Eximus **no** armor multiplier. Example, Eximus Heavy Gunner (Grineer, base 300 HP, b_lvl 8) at level 100: inspiration 300 × 6 × 103.95 = **187,109**; wiki 0.25 × 1200 × 5.76 × 279.39 = **482,789**. Non-Eximus Heavy Gunner at level 100 in the inspiration: 83,818.

### 2.5 Overguard (`getEnemyOverguard`, 7624-7636)
```
base = currEnemy.overguard.val if present, else (Eximus ? 12 : 0)
p = 1 + 0.0015 Δ^4 ;  q = 1 + 260 Δ^0.9
Δ < 45        -> p * base
45 ≤ Δ ≤ 50   -> q * base
Δ > 50        -> blend via eOGTrans(45,50,level), which always returns 1 here -> q * base
```
`eOGTrans` (7617-7623) returns 1 whenever `level − b_lvl > 50`, which is the only case in which it is called, so its broken smoothstep (it normalises twice through `tXOG`) is dead code. In effect: f1 below Δ=45, f2 from Δ=45 on, with no smoothstep. Wiki (WEDS §7.5): smoothstep over Δ 45-50. At Δ=44 the inspiration gives 12·f1 = 67,478; at Δ=45 it jumps to 12·f2 = 95,962 (the wiki gives 73,823 there). Overguard gets no Steel Path ×2.5 and no party multiplier. Changelog: "Minor adjustments have also been made to the formula for calculating the amount of overguard, increasing it" (guide.txt 2025-10-06).

### 2.6 Steel Path summary
Health and shield ×2.5 (before scaling), armor unchanged (matches U36, WEDS §8.1), overguard unchanged, no +100 level. Not modelled: the flat Steel Path base-health bonuses (Juggernaut +1,800 etc., WEDS §8.1).

---

## 3. Armor

### 3.1 Damage-reduction formula
```
getNetArmor(a)        = parseFloat(a)                       # 8145-8147 (no-op)
getDamageReduction(a) = sqrt(3·a) / 100                     # 8148-8150
```
`sqrt(3a)/100` equals `0.9·sqrt(a/2700)` (WEDS §3.1). There is no `a/3000` branch above 2700, because armor is clamped to 2700 first (8115-8116). Changelog: "DR formula now is sqrt(3*armor)" (guide.txt 2024-07-04). With 2700 → DR 0.9.

### 3.2 Direct hits on armor (`armorDmgWithDecreaseArmor`, 9801-9817)
```
A = currEnemyArmorVal * max(Fa, 0)                          # Fa = strip multiplier (3.3), 1 for the base tables
for each damage type t of the HEALTH map:
    armorDmg[t] = healthDmg[t] * (t == finisher ? 1 : (1 - DR(A)) * (1 - innateDR.armor))
    if 0 < armorDmg[t] < 1: armorDmg[t] = 1                 # per-type floor of 1
```
The armor layer is the health map (faction modifiers already applied) times the armor factor. Finisher bypasses armor. The per-type minimum of 1 is an inspiration rule with no wiki source.

### 3.3 Armor strip inside the queue (getModelQueueDmg, 10806-10822)
Only for the armor layer (`ka`). `d[A]` is the status timeline at hit A (stack counts, built by `timelineStatus`, 10093-10135):
```
Fa = 1
if Pa > 0 and puncture active:  Fa *= 1 - Pa * punctureStacks      # Pa = mod stat "decr_armor_by_punc"; no mod in the table defines it (only reader is 10369)
if corrosive active and not immune: Fa *= 1 - (0.20 + 0.06 * corrosiveStacks)
if (heat active or Napalm or reconifexHeat) and not heat-immune: Fa *= 0.5
if qa > 0:  Fa *= 1 - qa                                           # qa = getStatValFromMods("armor"): Corrosive Projection, 0.18 per aura (4883)
Fa = max(Fa, 0)
```
- Corrosive matches the wiki: 26% at 1 stack, 80% at 10 (WEDS §3). Max stacks are 10 + `incMaxStacks_Corrosion` (Emerald shard), also capped by `maxCountProc` (10093-10135, 10070-10082).
- Heat: a full 50% strip as soon as any Heat stack is active. The wiki says it ramps over 2 s and recovers over 6 s, scaled by Status Duration (WEDS §3 c). Not modelled.
- The "decrease armor" option (`decreaseArmorOpt`, a % of total armor for ability strips) is applied to **base** armor in `showEnemy` and to the capped/floored value in `correctEnemyLvl`. Corrosive Projection is then applied **multiplicatively** through `Fa`. The wiki says CP is **additive** with ability strips (WEDS §3 a).
- The timeline counts a hit's own procs at its own index (10110-10112), so a hit's own Corrosive or Heat proc lowers the armor that the same hit meets. See Open questions.
- `Fa` is stored as `decrease_armor_mult` (10823) and passed into DoT calculations, so DoTs use the armor strip at the **time of the inducing hit**, not at tick time.

### 3.4 Armor on DoTs (`getTickStatusDmg`, 9152-9204; `calcDmgOnStatusEff`, 10239-10253)
```
tick      = C * moddedBase * (1 + elementMods + addX) * d * mirage * dbl_mult      # C = 0.5, Blast 0.3 (9177)
health    = calcPercentAdd(tick, factionMod(type))
shield    = calcPercentAdd(tick, factionMod(type))                  # only if currShield > 0
overguard = type == Void ? tick*1.5 : tick                          # only if currOverguard > 0
armor     = (h' - DR(currArmor*max(Fa,0)) * h') * (1 - innateDR.armor),  h' = calcPercentAdd(health, factionMod(type))
```
On the armor layer the faction modifier is applied **twice** (9199-9200). See Suspected quirks. Slash bleed does not go through this function. It is `0.35 × moddedBase × factionDamageMod` with no faction type modifier and no armor (10913-10915), which matches the wiki (Cinematic damage).

`calcPercentAdd(b, m)` (8182-8185): `m > 1 → b·m; m falsy → b; else → b − b·m`. For m = 0.5 this gives 0.5b (correct). For any other value below 1 it would give `b·(1−m)` instead of `b·m`. The current tables only hold 0.5 and 1.5, so no number is wrong today.

---

## 4. Faction damage-type modifiers

`typeOfFaction` (5503-5518), looked up by exact faction string (`findTypeOfRes`, 9205-9211). Missing type → `false` → ×1.

| Faction key | ×1.5 | ×0.5 | Wiki (WEDS §2) | Difference |
|---|---|---|---|---|
| Grineer | Corrosive, Impact | – | Impact, Corrosive | – |
| Kuva Grineer | Corrosive, Impact | Heat | same | – |
| Sentient | Cold, **Magnetic**, Radiation | Corrosive | Cold, Radiation / Corrosive | inspiration adds Magnetic ×1.5 |
| Corpus Amalgam | Electricity | Blast | Electricity, **Magnetic** / Blast | inspiration lacks Magnetic ×1.5 |
| Murmur | Electricity, Radiation | Viral | same | – |
| Anarchs | Electricity, Impact | Radiation | same | – |
| Narmer | Toxin, Slash | Magnetic | same | – |
| Corpus | Puncture, Magnetic | – | same | – |
| Orokin | Puncture, Viral | Radiation | same | – |
| Infested | Slash, Heat | – | same | – |
| Infested Deimos | Blast, Gas | Viral | same | – |
| Zariman | Void | – | same | – |
| Scaldra | Corrosive, Impact | **Toxin** | Impact, Corrosive / **Gas** | inspiration resists Toxin instead of Gas |
| Techrot | Gas, Magnetic | Cold | same | – |
| (no entry) Stalker, Neutral, Tenno, Unaffiliated, UNKNOWN, Infestation, Arbiters of Hexis, "", " Infested", " Sentient" | – | – | Thrax/Void Angel → Zariman; Wolf of Saturn Six → Grineer; Acolytes neutral | Thrax, Void Angel and Wolf get no modifiers |

Where the modifiers apply:
- **Health and shield** direct damage: per type, after quantization (`reCalcWithNonCritBase`, 9255-9327: `calcPercentAdd(setQuantize(...), findTypeOfRes(type))`).
- **Armor**: inherits the health values, so faction × armor DR.
- **Overguard**: no faction modifier; Void ×1.5 only (9335-9346). Matches the wiki ("Overguard: Void").
- **DoTs**: see 3.4. Slash ticks get none.
- `getFactionMod()` (9104-9109) is the player's faction-damage mod (Bane/Roar), not the enemy modifier. It is applied to the base damage and again on DoT ticks (`oa` in 10392-10394 and in `getTickStatusDmg`), which matches the wiki's (1+F)² for DoTs.
- `elemRes` (enemy-specific resistances): applied to layer totals by `getTotalTypeOfArmorDmg` (9110-9119), to the toxin share (10336-10339) and to every DoT (`lb`, 10889-10893). It is **not** applied on the armor layer's direct hits: for armor, `g` is rebuilt from `armorDmgWithDecreaseArmor(Fa, currWeapon.health)` (11364-11374), which sums raw types.

---

## 5. Layers: health, shield, overguard, armor

### 5.1 Per-layer damage maps (`reCalcWithNonCritBase`, 9248-9347)
Four maps are built for each attack:
- `health`: if the enemy has shields **and** the attack has Toxin, only the Toxin entry; otherwise all types (9265-9283).
- `healthFull`: all types, always (9285-9299).
- `shield`: if the enemy has shields: all types except Toxin when the attack has Toxin (9301-9325).
- `overguard`: all types, no faction modifier, Void ×1.5 (9327-9346).
- `armor = armorDmgWithDecreaseArmor(1, health)`, `armorFull = armorDmgWithDecreaseArmor(1, healthFull)` (9344-9345).

So Toxin bypasses shields: direct Toxin goes to the health map while shields are up. The wiki exceptions (Treasurer, Hounds; WEDS §4.2) are not modelled: the code has no per-name special cases (no match for Treasurer or Hound).

`createDmgQueue` (12555-12715) runs `getModelQueueDmg` once per layer: `health` (plus `healthFull` if shields), `armor` (plus `armorFull` if shields) when armor > 0, `shield` when shield > 0, `overguard` when overguard > 0. The third argument `g` (layer total) comes from `getTotalTypeOfArmorDmg(layer, attack)` (12553-12555, 12575-12579), which applies `elemRes`.

### 5.2 Per-hit layer damage (getModelQueueDmg, default branch 11960-11963)
```
curr_dmg = (g · Jb + Ac) · (1 − innateDR[layer]) · Ta · hc
Jb = multishotTier · Ga(vulnerability) · ta(headshot dmg) · Ma(headshot mult) · crit · I(Viral) · Q(Magnetic)
     · melee/combo/slam/strength/buff/first-last-bullet/... multipliers          (10834-10854)
Ta = 1 and hc = 1 in the current build (unique DR disabled, section 8)
toxinDmg = curr_dmg · W,  W = (layer Toxin · (1−elemRes.toxin)) / layer total   (10336-10341, 11964)
```
- **Viral** `I = 2 + 0.25·(stacks − 1)` (10780). Active only on health and armor layers (`Kd` false for shield and overguard, 10476-10482). External Viral stacks (`externalVirus`) add stacks, capped by Viral max 10 and `maxCountProc` (10781-10797). The changelog says "Fixed a bug where the viral status effect multiplier was affecting overguard" (2025-07-30). Matches the wiki: ×2 at 1 stack, ×4.25 at 10.
- **Magnetic** `Q = 2 + 0.25·(stacks − 1)` (10802-10804). Only on shield and overguard layers (`Vd`, 10483-10488). Matches the wiki (Shield.wiki line 363: +100%, +25% per stack, 325% total).
- Viral and Magnetic also multiply Napalm DoT values `jc`/`ac` (10800, 10804).
- **Overguard-only multiplier** `Cb` = `mult_overguard` mod stat (Secondary Fortifier) when on the overguard layer (10406-10408).
- **innateDR** is multiplied once on direct damage (11961-11963) but **squared** on most DoTs: `V = (1 − innateDR[layer])²` (10885) is used for Slash, Heat, Electricity, Gas and Blast ticks (10917, 11078, 11159, 11230, 11302). Toxin ticks use it once (10999-11000). On the armor layer, `getTickStatusDmg` applies `(1 − innateDR.armor)` again (9202), so Heat/Electricity/Gas/Blast ticks against armor get it **cubed**.
- `innateDR.status` (Archons 0.5) multiplies every status tick as `Ra` (10896, 10966, 11040).
- **Shield DoT ×3**: `Ob = (layer == shield) ? 3 : 1` (10576) multiplies every DoT tick value on the shield layer (Slash, Toxin (shield toxin is skipped anyway), Heat, Electricity, Gas, Blast, Debilitate, Toxic Lash; 10921, 11001, 11082, 11161, 11232, 11304, 11439, 11470, 11620, 11651, 11799, 11829). No wiki source found. See Open questions.

### 5.3 Shield gate, overflow, regeneration
- The enemy shield gate (0.1 s, 5% to health; Shield.wiki line 341) is **not** modelled anywhere.
- Damage overflow is discarded: `applyDamageLayer` (13805-13809) returns `{value, leftover}` and every caller keeps only `.value` (13733-13798). A hit that breaks overguard or shield does nothing more that instant.
- No shield regeneration, no Magnetic shield/overguard-break Electricity proc (Damage.wiki line 174). No reads of 3%-of-max-shield logic were found in 10311-12015.

---

## 6. Immunities and status stack limits

- `enemyImmunStatus` is checked per status at roll time (`getCurrTypeChanceForStatus`, 10060-10069: an immune type's weight is removed from the proc weights) and before each effect: Viral (10476), Magnetic (10483), Cold (10489), Heat (10493, 10770-10771), Puncture (10651, 10808-10810), Corrosive (10811-10815), Slash (10904), Toxin (10988), Heat DoT (11059), Electricity (11145-11148), Gas (11219), Blast (11291). `"all"` disables every status effect, status-damage mods (`Nb`, 10520-10521) and base-per-status bonuses (9216, 9371).
- `maxCountProc` (enemy `maxProcStacks`, 0 = unlimited):
  - In the roll (9997-10043): a new proc of type T is dropped when this hit already holds `≥ maxCountProc` procs of T (Corrosive: `maxCountProc + incMaxStacks_Corrosion`). This counts within one hit, not across hits.
  - In the timeline (`addTick` 10070-10082, `timelineStatus` 10093-10135): active stacks are capped at `max(min(statusDuration.max (+Corrosion bonus), maxCountProc (+Corrosion bonus) or ∞), 1)`. `statusDuration.max` (5534-5584): Impact 5, Puncture 5, Cold 10, Corrosive 10, Gas 10, Magnetic 10, Radiation 10, Viral 10, Tau 10, Void 1, Microwave 1; Slash, Heat, Electricity, Toxin, Blast unlimited.
  - In TTK, DoT ticks are scheduled per proc in the hit's `statuses` list (section 9), so Slash/Toxin/Heat stacks are limited only by the per-hit roll cap, not by stacks already on the enemy.
- Corpus "Target"/Ranger units: `elemRes` 0.75 to Cold, Blast, Magnetic and Viral (damage and DoT); Hyekka/Hyekka Master 0.8 to Heat.

---

## 7. Optimizer use of TTK (WFOpt, 15252-end; summary only)
- The TTK metric score `fa()` (16020-16049): not killed → `−(queueLength + EHP/dps)` with `EHP = currHealth + currShield + currOverguard` (raw pools, no armor); killed at t ≤ 0 → `0.001 + ln(1 + avgHit/EHP)` (one-shot margin); otherwise `−ttk` plus a small tie-break by hit size. Display: "not killed, EHP/DPS ~x s" (15371-15378).
- `Hb()` (15766-15797) takes the **last** TTK sample of `TTK_Queue[attack][defense]`, where defense is the layer row the user selected (health/armor/shield/overguard).

---

## 8. Unique damage reduction (13070-13193) — disabled in this build

`showEnemy` replaces `currEnemy.unique` with `"notset"` (7690; also in `min.js`: `currEnemy.hasOwnProperty("unique")&&(currEnemy.unique="notset")`). Changelog: "2025.10.15 ALL unique damage reduction modifiers for enemies have been disabled", "2025.10.17 Enabled old damage reduction for unique enemies", "2025.10.18 Unique damage reduction modifiers for enemies have been disabled" (guide.txt lines 276-277). Effects:
- `swithTypeUniqueEnemyDR` (13142-13193) hits no `case` and returns 1.
- `S = currEnemy?.unique ? true : false` (10347) is **true** for the 114 unique enemies (non-empty string), but every specific flag (`gc`, `Yb`, `Yc`, `Zc`, `Ib`, `xc`, `yc`, `Zb`, `wb`, `xb`, `tb`, `Xc`; 10539-10556) is false, so the default branches run and `Ta = 1`, `hc = 1`.
- The UI still shows the "unique damage reduction ... will not always be fully taken into account" notice (7752).

The functions are still in the bundle. Their formulas, for reference (b = the "metric" argument):

| Function | Formula | Notes |
|---|---|---|
| `demolisherDR(b)` 13070-13077 | b≤1000: 1; ≤2500: 0.8+200/b; ≤5000: 0.7+450/b; ≤10000: 0.4+1950/b; ≤20000: 0.2+3950/b; else 0.1+5950/b | multiplier |
| `jugulusDR(b)` 13098-13105 | identical to demolisherDR | used for jugulus, saxum |
| `jugulusDRProc(b)` 13106-13113 | b≤1562.5: 0.64; ≤3906: 0.512+200/b; ≤7812.5: 0.448+450/b; ≤15625: 0.256+1950/b; ≤31250: 0.128+3950/b; else 0.064+5950/b | jugulus status ticks (`p=true`) |
| `amalgamDR(b)` 13091-13097, `bursaDR(b)` 13125-13131 | b≤1000: 1; ≤6000: 0.8+200/b; ≤10000: 0.7+800/b; ≤30000: 0.5+2800/b; else 0.2+11800/b | amalgam, amalgam-machinist, empyrean-corpus, bursa |
| `orphixDR(b)` 13118-13124 | b≤1000: 1; ≤2500: 0.7+300/b; ≤5000: 0.5+800/b; ≤10000: 0.2+2300/b; else 0.02+4100/b | multiplier |
| `acolytesDR(b)` 13085-13090 | b≤4000: 0.75; ≤10000: 0.6+600/b; ≤30000: 0.4+2600/b; else 14600/b | metric from `acolytesMetric` (10283-10294): `dps = ms·speed/(t/(crit−t)+1)·dmg·(shotgun?0.5:1)`, `t = 1 − 1/max(1,tier)`; also replaces the crit multiplier by `((1+0.5(crit−1))/crit)·(1+tier(crit−1))` |
| `eidolonDR(b, speed)` 13078-13084 | thr = 108/(speed·multishot); b≤thr: 0.4·b; else 0.02·b + thr | returns **damage**, not a multiplier; input includes ×2 body and ×2 (non-headshot) factors (11498-11512) |
| `lephantisDR(b, speed)` 13114-13117 | b≤450: 1; else speed/0.25 > 1 ? 0.1 : 0.1+450/b | lephantis, hemocyte |
| `archonDR(b)` 13135-13138 | 1/(1 + b·multishot/460000) | via switch |
| `getArchonDR(dmg, ms, shotgun)` 13265-13268 | x = dmg·ms·(shotgun?0.7:1)/460000; return x/(x+1)·460000/ms | soft cap per hit at 460000/ms |
| `suzerainDR` 13132-13134 | 1 | |
| `demolisherNecramechDR` 13139-13141 | 0.5875 constant | used as `hc` for all damage of Demolisher Voidrig (10557, 11966) |
| Xata cap (11406, 11587) | Voidrig/Dedicant: Xata hit capped at round(176000/multishot) | |

For the Xc group the metric is a per-second figure: `mb = (g/factionMod)·speed·multishot·headshotMult·(1−innateDR)` (11378-11394). Keys with no DR function: crewmans-rangers, sentient, raknoid, hyekka, tusk-thumper, juggernaut, condrix, vomvalyst, lich, necramech, stalker, sister, prosecutor, prelate, treasurer, efervonTank, dedicant (only the Xata cap), obsolyte, babau. WEDS §9 documents the real post-U40 attenuation, which was removed from Demolishers, Jugulus and Amalgams (Damage_Reduction.wiki 327-341) but remains on bosses (Archons, Eidolons, Acolytes...).

---

## 9. Time to kill (`calcTTKWithStatusUnified`, 13593-13804; `applyDamageLayer`, 13805-13809)

`applyDamageLayer` is the 4-line function at 13805-13809. Lines 13810-15252 are the jQuery `ready` handler (UI events, the "new queue" button), not damage code.

### 9.1 Inputs (call sites 12697-12715)
For each **attack** of the weapon separately (no combined TTK across attacks):
```
pools = {health: currHealth, shield: currShield, overguard: currOverguard}
Health TTK: layers = [OG records, shield records, health records, healthFull records]
Armor TTK : layers = [OG records, shield records, armor records,  armorFull records]   (only if armor records exist)
calcTTKWithStatusUnified(layers, queueLength, pools, statusDuration, attackHasToxin)
```
The health-TTK result is pushed into TTK_Queue[attack].health, .shield and .overguard (all the same value). The armor TTK goes to .armor. Both use the same health pool; the "health" row is the TTK as if the enemy had no armor. `queueLength` = simulated window in seconds, default 20 (5496), UI 1-60 (14864, 16603).

Each record (one per hit, from getModelQueueDmg) carries: `timeM` (hit time from the shot schedule, which already includes fire rate, bursts, charge and reloads; 10591-10593), `curr_dmg`, `toxinDmg`, `otherDmg` (Napalm + reconifex heat), `dmgOnStatusEff`, `dmgOnStatusEffToxin`, `xataTotal`, `toxicLash{direct_hit,tick_dmg}`, `statuses[]` (procs of this hit), `status[type]{d, nc, proc_d, proc_nc, instance_d}` (per-tick DoT value on that layer), `isNC` (use the non-crit `nc` values).

### 9.2 Algorithm
```
phase = OG>0 ? 0 : shield>0 ? 1 : 2                 # 13602
direct(rec) = curr_dmg + otherDmg + dmgOnStatusEff + xataTotal + toxicLash.direct_hit
events: next hit (index F), DoT tick heap H (sorted by time), next Heat tick M
loop while hits remain or H non-empty or M finite:
    tHit = timeM of hit F in the CURRENT phase's list (∞ if none or > queueLength)
    if tHit <= min(H[0].time, M):                     # hits win ties
        phase 0: OG     -= direct(OGrec)
        phase 1: shield -= max(0, curr−toxinDmg) + otherDmg + max(0, dmgOnStatusEff−dmgOnStatusEffToxin) + xataTotal   (shield rec)
                 health -= toxinDmg + dmgOnStatusEffToxin + toxicLash.direct_hit                                       (health rec, toxin-only map)
                 (+ shield rec's toxicLash.direct_hit too when attackHasToxin && enemy has shield)
        phase 2: health -= direct(healthFull rec) (+ shield rec's toxicLash.direct_hit when attackHasToxin && shielded)
        schedule DoTs of this hit (below); if health <= 0 return tHit; advance phase
    elif M <= H[0].time:                              # merged Heat tick
        dmg = Σ over every Heat source in Q of the source hit's heat value on the CURRENT layer
        apply to the current phase's pool; if health <= 0 return M
        M += 1 s; if M > heatExpiry or M > queueLength: clear Q, M = ∞
    else:                                             # one DoT tick
        layer = phase; if phase == shield and type is Toxin/ToxicLash: layer = health
        dmg = source hit's status[type].d (or .nc) on that layer's record
        apply; if health <= 0 return tick time; advance phase
return null                                           # "not killed" within queueLength
```
- Layer damage subtracts, and the excess beyond the current pool is lost (`applyDamageLayer`). Phase advances OG → shield (if > 0) → health (`ra`, 13719-13722).
- **Toxin in shield phase**: direct Toxin and Toxin ticks hit health; health can reach 0 with shields still up (13740-13756, 13786-13787). Toxin does not bypass overguard (phase 0 applies the whole hit to OG). Matches the wiki.
- **DoT scheduling** (`ka`, 13648-13716) at each hit time S, for every status type in the hit with `statusDuration[type].ticks > 0` except Heat and ToxicLash, and with a positive tick value on some layer: one schedule per proc of that type in the hit (minimum 1); ticks at `S + delay + i·(duration/ticks)`, i = 0..ticks−1, kept only if ≤ queueLength. Tick values are looked up from the source hit's record **for whatever layer is current when the tick lands**.
  - Uses base `duration`, `ticks`, `delay` from `statusDuration` (5534-5584), **not** `curr_duration`, so Status Duration mods do not lengthen DoTs in TTK (they do in the DPS totals: `curr_duration = parseInt(duration·(1+SD))`, 9085-9093).
  - Slash, Toxin: 6 ticks, delay 1 → t+1..t+6. Electricity, Gas: 6 ticks, delay 0 → t+0..t+5. Blast, Corrosive, Viral, Magnetic, Cold, Puncture, Radiation, Void: ticks 0 → never scheduled, so the Blast proc's delayed damage (computed at 11291-11355) never enters TTK.
  - Gas: at most `statusDuration.Gas.max` = 10 concurrent Gas instances; new ones are skipped (13682-13687).
  - Toxic Lash: duration 7.5, ticks 7.5, delay 1; the loop `i < 7.5` gives **8** ticks (13706-13713).
- **Heat** (13691-13704, 13762-13782): every Heat proc in a hit adds a source to Q (one per proc, plus an `instance` source for reconifex-type instant heat). Expiry `n = S + 6` is refreshed by each new Heat hit. If no Heat tick is pending, the first is at `S + 1`. Each tick sums the per-tick values of **all** sources in Q, every 1 s, until the tick time passes n; then Q is cleared. This models the wiki's shared refreshing Heat timer (WEDS / wiki-damage-types §Heat). New sources join at the next pending tick.
- Target-side multipliers inside a tick (Viral, Magnetic, armor strip) are those **at the inducing hit's time**, baked into the record (section 5.2), not re-evaluated at tick time as the wiki requires.

### 9.3 Sampling and display
- One `reCalc` = one random simulation of procs and crits → one TTK sample per attack and defense row.
- "New queue" button (14273-14484): calls `reCalc` in a `setInterval(…, 0)` loop for 5 s, then shows the **median** of all TTK samples (`null` → ∞ → "not killed"), formatted with 3 decimals. DPS is also the median of the per-run values, labelled "Avg" (14319-14338).
- Changelog for TTK: "Added experimental Time To Kill (TTK) calculation" (2025-06-26), "Improved calculation of TTK taking into account the toxin bypassing shields" (2025-07-11), "Changes have been made to the TTK calculation. The calculation is now more accurate" (2026-11-17).

---

## 10. Enemy list (`samples/enemies-min.json`, 498 entries)

Counts and ranges per faction string (values are base values at `b_lvl`; health −1 and 1/1/1 placeholders included):

| Faction (verbatim) | Count | Health min-max | With shield | Shield min-max | With armor | Armor min-max | eximusOff | b_lvl set | unique set |
|---|---|---|---|---|---|---|---|---|---|
| `Corpus` | 154 | 1-12000 | 147 | 1-30000 | 76 | 1-500 | 23 | 6 | 38 |
| `Grineer` | 118 | -1-15000 | 18 | 50-1400 | 116 | 5-2750 | 24 | 10 | 11 |
| `Infested` | 49 | 20-20000 | 3 | 1000-1200 | 19 | 25-500 | 15 | 4 | 10 |
| `Narmer` | 33 | 35-30000 | 16 | 50-5000 | 18 | 5-500 | 4 | 0 | 3 |
| `Sentient` | 26 | 300-65000 | 14 | 200-70000 | 19 | 25-300 | 15 | 2 | 23 |
| `Kuva Grineer` | 25 | 1-2500 | 1 | 1-1 | 25 | 1-500 | 2 | 8 | 3 |
| `Orokin` | 19 | 35-4500 | 14 | 50-1500 | 9 | 5-500 | 1 | 4 | 1 |
| `Stalker` | 9 | 750-5500 | 9 | 200-2500 | 9 | 50-300 | 9 | 0 | 8 |
| `Infested Deimos` | 8 | 100-8000 | 0 | - | 6 | 50-600 | 4 | 0 | 3 |
| `Neutral` | 8 | 2-8000 | 1 | 500 | 8 | 20-250 | 0 | 0 | 0 |
| `Corpus Amalgam` | 7 | 500-650 | 7 | 100-400 | 0 | - | 0 | 0 | 7 |
| `Anarchs` | 7 | 75-325 | 0 | - | 7 | 200 | 0 | 0 | 0 |
| `Scaldra` | 6 | 50-18000 | 0 | - | 6 | 10-500 | 1 | 0 | 2 |
| `Techrot` | 6 | 80-10000 | 6 | 50-500 | 0 | - | 0 | 0 | 2 |
| `Murmur` | 5 | 500-50000 | 0 | - | 2 | 100-175 | 2 | 1 | 2 |
| `Tenno` | 4 | 100-2000 | 3 | 80-500 | 4 | 20-500 | 0 | 0 | 0 |
| `""` (empty) | 4 | -1-1700 | 0 | - | 3 | 200 | 3 | 0 | 0 |
| `UNKNOWN` | 3 | 3200-25000 | 1 | 100 | 3 | 200-350 | 3 | 0 | 0 |
| `Unaffiliated` | 2 | 800-1500 | 0 | - | 2 | 200-300 | 1 | 0 | 0 |
| `Infestation` | 2 | 1000-1200 | 0 | - | 2 | 200 | 0 | 0 | 0 |
| `Arbiters of Hexis` | 1 | 35 | 1 | 50 | 0 | - | 1 | 0 | 0 |
| `" Infested"` | 1 | 5000 | 0 | - | 1 | 25 | 1 | 0 | 0 |
| `" Sentient"` | 1 | 5000 | 1 | 2000 | 1 | 200 | 1 | 1 | 1 |

Small factions in full (health/shield/armor):
- Stalker: Acolytes, Angst, Mania, Malice, Misery, Torment, Violence (5500/2500/50, innateDR health/armor 0.625, maxProcStacks 4), Shadow Stalker (950/200/300), Stalker (750/200/300).
- Corpus Amalgam: Alkonost (650/300/0, immune all), Heqet (500/400/0), Kucumatz (500/100/0), Osprey (500/300/0), Satyr (600/150/0), MOA (500/300/0), Machinist (600/300/0, innateDR shield 0.5).
- Infested Deimos: Ancient Healer (400/0/0), Therid (1200/0/0, Viral-immune), Jugulus (1600/0/300), Tendril Drone (100/0/150), Genetrix (8000/0/600), Juggernaut (900/0/200), Carnis (800/0/50, Viral-immune), Saxum (750/0/70).
- Anarchs (all armor 200): Arcus 100, Gladius 175, Grineer Lancer 125, Grineer Trapper 100, Libritor 325, Capsarii 75, Tenebra 100.
- Scaldra: H-09 Efervon Tank (18000/0/100, maxProcStacks 4), Jaeger (125/0/100), Flayer (50/0/10), Barbican (125/0/100), Eradicator (325/0/500), Dedicant (2000/0/500, overguard 22).
- Techrot: Galliflex (100/100/0), Volatile Galliflex (80/50/0), Matmas (300/150/0), Skuzzi (225/125/0), Obsolyte (400/200/0), Babau (10000/500/0, immune Toxin and Viral).
- Murmur: The Fragmented Suzerain (50000/0/0, immune Viral and Cold), The Anatomizer (6000/0/0, b_lvl 10), The Hollow Vein (7500/0/0), Rogue Culverin (500/0/175), Demolisher Voidrig (4500/0/100, maxProcStacks 4, party).
- Tenno: Arcane Machine (2000/500/25), Bailiff Defector (300/0/500), Devotee (100/100/100), Specter (Enemy) (200/80/20).
- Neutral: Ashen Kuaka (Old) (2/0/50), Desert Skate (250/0/200), Feral Kavat (300/0/175), Feral Kubrow (300/0/200), Maroo (50/500/20), Tomb Protector Effigy (5500/0/250), Tomb Guardian (1700/0/150, Slash-immune), Tomb Protector (8000/0/250).
- UNKNOWN: Thrax Legatus (3200/0/200, OG 10), Void Angel (25000/100/350, maxProcStacks 4), Thrax Centurion (3600/0/200, OG 15).
- Empty faction: Narmer Deacon (−1/0/−1), The Grustrag Three/Leekter, /Shik Tal, /Vem Tabook (1700/0/200 each).
- Others: Unaffiliated: Saturn Six Fugitive (800/0/300), Wolf of Saturn Six (1500/0/200, immune all). Infestation: Zealot Proselytizer (1200/0/200), Zealot Herald (1000/0/200). Arbiters of Hexis: Arbitration Shield Drone (35/50/0, immune all). `" Infested"`: Phorid (5000/0/25). `" Sentient"`: Ropalolyst (5000/2000/200, b_lvl 17).

Notable special entries (all from the JSON): Eidolon Teralyst 50000/50000/125, Gantulyst 55000/60000/130, Hydrolyst 65000/70000/150 (immune all); Archon Amar/Boreal/Nira 30000/0/25 (innateDR status 0.5, health 0.2, armor 0.2; maxProcStacks 4); Profit-Taker Orb 7000/30000/150 (immune all, innateDR shield 0.5); Exploiter Orb 12000/0/200 (immune all); Treasurer 2500/150/0 (innateDR health 0.98, shield 0.99); Hemocyte 5000/0/175 (innateDR 0.7, immune all, b_lvl 10); Lephantis 3000/0/0 (innateDR 0.6, immune all); Zealoid Prelate 9000/0/0 (maxProcStacks 10); Kuva Lich and Sisters of Parvos 1/1/1 (placeholders); Corrupted Heavy Gunner 700/0/500 (Orokin, b_lvl 8); Demolishers: innateDR health/armor 0.2 (Corpus ones shield 0.5), Radiation-immune, Infested ones Viral-immune.

---

## 11. Worked example: `research/test-cases/user-example-furis.json`
Corrupted Heavy Gunner (Orokin, base 700 HP / 0 shield / 500 armor, b_lvl 8), level 1000, Steel Path, not Eximus, no armor option. Δ = 992, s = 1.
- Health: 2.5 × 700 = 1750; HealthMult = 1 + 10.7331 × 992^0.685 = 1212.5371; health = round(round(1750 × 1212.5371 × 100)/100) = **2,121,940** (8108-8111). WEDS §10 gets 2,121,953.5 with the float32 blend; the difference is 0.0006%.
- Armor: 500 × (1 + 0.4 × 992^0.75) = 35,852 → capped **2700** → DR **0.90** (8112-8116, 8148-8150). Steel Path does not touch armor.
- Shield 0, overguard 0 (no base OG, not Eximus).
- Faction modifiers for Orokin: Puncture ×1.5, Viral ×1.5, Radiation ×0.5.

---

## Suspected inspiration quirks

1. **All unique damage reduction is disabled** by `currEnemy.unique = "notset"` (7690). Archons, Eidolons, Acolytes, Lephantis etc. take full damage; the DR functions are dead code. Deliberate per changelog 2025-10-18.
2. **Eximus scaling** ignores faction curves (uses Default/Sentient curves), keys the step factor on absolute level instead of Δ, has no `max(1.1·base, c·(base+900)·k)` base bonus, and multiplies armor by up to 1.625 (7637-7686). The Eximus Heavy Gunner example is 187k in the inspiration vs 483k on the wiki at level 100.
3. **Overguard has no smoothstep**: f1 below Δ=45 and f2 from Δ=45 on, a jump of about +30% at Δ=45 (7624-7636). `eOGTrans` is broken but unreachable.
4. **Techrot curves differ from the wiki**: health f1 0.015Δ^2.1 (wiki 0.02Δ^2.12), f2 13.64Δ^0.72 (wiki 15.0998Δ^0.70); shield f2 3.4106Δ^0.76 (wiki 3.5) (8078-8083).
5. **Missing faction curves**: Scaldra (wiki: Grineer class), Corpus Amalgam (wiki: Corpus class), Anarchs (wiki module: Orokin class) fall to the default curve. Sentient/Neutral/Murmur use shield f2 = 2.0, the wiki gives 1.6 for Sentient (8084-8094).
6. **Faction table errors**: Sentient gets an extra Magnetic ×1.5; Corpus Amalgam lacks Magnetic ×1.5; Scaldra resists Toxin instead of Gas (5503-5518). Thrax, Void Angel (wiki: Zariman) and Wolf of Saturn Six (wiki: Grineer) get no modifiers. `" Infested"` (Phorid) and `" Sentient"` (Ropalolyst) have a leading space and miss both their curves and modifiers.
7. **Armor strip option inconsistencies**: with Steel Path on, the "decrease armor" % is only applied when the result lands on the 2700 cap or 200 floor (7699-7703 vs 8115-8116). Without Steel Path, a value that the strip pushes below 200 is reset to `200·(1−d)` instead of `scaled·(1−d)`. Corrosive Projection multiplies with ability strips (wiki: additive).
8. **Heat armor strip is an instant 50%** whenever any Heat stack is active (10815); the wiki ramps it over 2 s and recovers over 6 s.
9. **A hit's own Corrosive/Heat proc reduces armor for that same hit** (timeline counts the proc at its own index, 10110-10112; used at 10810-10815). Needs a game check.
10. **innateDR is squared on Slash/Heat/Electricity/Gas/Blast ticks** (10885) and cubed against armor (9202 plus 10885), but applied once on direct hits and Toxin ticks.
11. **Faction modifier applied twice on armor-layer DoTs** (`calcPercentAdd` twice, 9199-9200).
12. **DoT ticks on shields ×3** (`Ob`, 10576). No source found.
13. **`elemRes` ignored on armor-layer direct hits** (11364-11374 rebuilds the total from raw types).
14. **Armor-full records may use the toxin-only health map**: `armorDmgWithDecreaseArmor(Fa, currWeapon.health)` (11365) is hard-coded to `health`, also when the call is for `armorFull`. On a shielded, armored enemy hit by a Toxin weapon, armor-TTK's post-shield phase may then see only Toxin damage. Needs a runtime check.
15. **No shield gate, no overflow**: damage beyond the current layer is discarded (13805-13809); the 0.1 s / 5% gate is absent.
16. **TTK ignores Status Duration** (uses base `duration`/`ticks`, 13675-13681) and ignores Blast procs (ticks 0). Electricity and Gas first ticks land at proc time (delay 0).
17. **Toxic Lash gets 8 ticks** in TTK (`i < 7.5`, 13706-13713) against a 7.5× multiplier in the DPS totals (12002, 12208).
18. **Possible double count of Toxic Lash direct hit** in the health phase when the attack has Toxin and the enemy has shields (13757-13759 add the shield record's `toxicLash.direct_hit` to `h(healthFull)`, which already includes its own).
19. **TTK is per attack**: a weapon with a direct hit and an explosion gets two separate TTKs; neither includes the other's damage (12697-12715).
20. **Target-side multipliers on DoTs are frozen at the inducing hit** (Viral/Magnetic/armor strip baked into the record) instead of evaluated per tick.
21. **"Avg" label shows the median** in the queue summary (14319-14338).
22. **Placeholder enemies**: Kuva Lich / Sisters of Parvos 1/1/1; Narmer Deacon health −1 and armor −1 (would kill on the first hit or show negative values).
23. **Eximus for b > 100 uses the pure high curve** even when Δ < 80 (7666, 7683, 7650).
24. `calcPercentAdd` treats a modifier `m < 1` as "reduce by m" (8182-8185). Harmless with today's 0.5/1.5 values; wrong for anything else (e.g. 0.75 would act as 0.25).

## Open questions

1. What is `Ob = 3` (shield-layer DoT ×3, 10576) meant to model? Is there an in-game rule that status damage against shields is tripled?
2. Does a hit's own Corrosive/Heat proc affect that hit's armor in game (quirk 9)? The wiki says procs apply after the hit's damage.
3. Is quirk 14 (`currWeapon.health` hard-coded in the armorFull pass) reachable in practice? Needs a runtime test with a shielded, armored enemy and a Toxin weapon.
4. Is quirk 18 (extra `toxicLash.direct_hit` in the health phase) intended to route the shield-blocked Toxic Lash hit to health, or a double count?
5. `decr_armor_by_punc` (10369) is read but no mod in the table sets it. Is it reserved for a future mod?
6. The game's rounding of scaled health/armor (inspiration: double `round` to 0.01 then integer; gadget: `floor`). Which matches the game?
7. Should Steel Path multiply overguard or Eximus base overguard (inspiration: no; WEDS §10: UNVERIFIED)?
8. Does overflow from a shield- or overguard-breaking hit carry over in game? The inspiration discards it; WEDS open questions 10-11 ask the same.
9. `queueLength` cuts DoT ticks scheduled after the window; TTK returns `null` instead of extrapolating. Should our TTK extend past the window (the optimizer already extrapolates as EHP/DPS)?
