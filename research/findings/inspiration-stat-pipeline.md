Status: draft

# Inspiration stat pipeline: base stats + mods -> final per-hit stats

Source: `research/external/inspiration/min.pretty.js` (abbreviated **L** + line numbers below). Function names are the
real top-level names; local variable letters are the minified ones and are given only where it helps a reviewer.
Game-truth comparisons cite the sibling file `research/findings/wiki-core-damage-formula.md` (abbreviated **WCF §n**) and
wiki pages cached in `research/external/cache/wiki/`.

Notation used in formulas:
- `Σmods(k)` = `getStatValFromMods(k)` (sum over equipped mods, see §1).
- `cust(k)` = `getCustomStat(k)` (user "custom stats" panel, L13262-13264; percent inputs are divided by 100 on entry, L15224-15240).
- `uniq(k)` = `getStatFromUnique(k)` (weapon-level `unique` block, L8782-8789).
- `aUniq(k)` = `getStatFromAttackUnique(k, attack)` (attack-level `unique` block, L8776-8781).
- `zoom(k)` = `getStatFromZoomLevels(k)` (L9095-9103).
- `inc(k)` = `getStatFromIncForm(isIncarnonAttack(attackName), k)` (L8790-8797).
- `isMaxCond` = option "all mod conditions" (`optAllModsConds`, default **true**, L5461, L12036-12038).
- `T(atk)` = `getTotalBaseDmg(i)` = Σ of `currWeaponInitial.attacks[i].damage` (L9792-9797). This is **after** the Incarnon flat bonus and the progenitor element have been written into `currWeaponInitial` (§3) and **before** mod elements.

---

## 0. Top-level order of operations (`reCalc`, L9638-9778)

```
reCalc():
  invalidateModStatsCache()                                     L9639
  flat = Σmods(flat_change_dmg) + cust(flat_change_dmg)
         + uniq(flat_change_dmg_by_strength) * strengthMult       L9640-9643
  if flat != 0: addDmgToBase(flat)                               L9644-9645
  else:
     if type not in {Zaw Component, Kitgun}: addDmgToBase(0)      L9646-9648
     if not (Zaw | Kitgun | Exalted | tags Tenet/Kuva Lich/Coda):
        currWeaponInitial.attacks = clone(currInitialAttacks)     L9649-9657   (see quirk Q1)
  if Σmods(impactToPuncture) > 0: swapDmgType(impact -> puncture) L9658-9659
  addWeaknessDmgType()                                           L9660   (faction-dependent element mods)
  newElements()        -> element combination, then setAdditionalMods()   L9661 / L12902-13069
  reCalcWithTypeBase(false) -> base-damage mods, CO, physical mods, per-layer quantized maps   L9662
  store currWeapon.health / healthFull / armor / armorFull / shield / overguard; createDmgQueue()  L9663-9672
  (rest of reCalc only serialises the build into the URL, L9673-9777)
```

The direct damage of one hit used by the simulator is `g = Σ_type currWeapon[layer][attack][type] × (1 − elemRes)`
(`getTotalTypeOfArmorDmg`, L9110-9119; called by `createDmgQueue` L12555-12571) multiplied by the per-hit factor `Jb`
inside `getModelQueueDmg` (L10860-10880; §12). So **the per-type maps from `reCalcWithTypeBase` are the "final per-hit
stats" for damage**, and `setAdditionalMods` produces the final crit/status/fire-rate/multishot/etc. stats.

---

## 1. Mod stat aggregation (`modStatsCache`, `rebuildModStatsCache`, `getStatValFromMods`)

`modsPanel` is a 30-slot array (L5449-5452). Slot meaning (from index.html `data-id` attributes and L8359-8384, L9697):
0-7 weapon mods, 8 exilus, 9 stance, 10 weapon arcane, 11-12 warframe arcanes, 13-16 warframe mods, 17 aura,
18-22 Incarnon evolution perks (`showIncMod`, L7566-7579), 23-27 archon shards, 28-29 extra (archwing/arcane) slots.
**Slot index order is also the element-combination order** (§4).

`rebuildModStatsCache` (L8668-8744), for every non-empty slot with `actions`:
1. `setMult = mod.set ? mod.set[getCountSetMods(set.name)] : 1` (L8713-8715; `getCountSetMods` L8514-8518 counts equipped mods with the same set name; e.g. Sacrificial `{1:1, 2:1.25}`, L165).
2. Choose source object and add every **numeric** key × setMult into `sums`:
   - `isMaxCond && actions.WITH_COND`:
     - if `WITH_COND.FOR_WEAPON`: helper `d(sums, WITH_COND.FOR_WEAPON, WITH_COND, setMult)`;
     - else helper `g(sums, WITH_COND, actions, setMult)`.
   - else if `actions.FOR_WEAPON`: `d(sums, actions.FOR_WEAPON, actions, setMult)`;
   - else `g(sums, actions, {}, setMult)`. (L8716-8722)
   - `g(h, primary, fallback, m)` (L8689-8696): add every numeric key of `primary`; add numeric keys of `fallback` **only if not present in `primary`**. So `WITH_COND` values **replace** (not add to) the unconditional value of the same key. Example: Incarnon "Haven Foray" `{flat_change_dmg: 28, WITH_COND: {flat_change_dmg: 58}}` gives 58, not 86 (samples/weapon-Furis.json). Galvanized Diffusion `{multishot: 1.1, WITH_COND: {multishot: 2.3}}` gives 2.3 (L868).
   - `d(h, forWeapon, base, m)` (L8675-8688): for **each** of `currWeapon.categories` that has an entry in `forWeapon`, add its numeric keys (several matching categories add up); then add numeric keys of `base` not seen in any matched category.
3. Incarnon-form tables (L8723-8724), helper `p` (L8697-8708): `incForm += first truthy of [actions.INCARNON, actions.WITH_COND, actions]` per key; `normForm` likewise with `actions.NORMAL`. **Not gated by isMaxCond and not set-scaled.**
4. Element table (L8725-8740): for every key in `WITH_COND.element` (only if isMaxCond) ∪ `element`: value = `WITH_COND.element[k]` if truthy else `element[k]`; summed into `elements` (not set-scaled). Used only by `getStatValFromModElements` (L8749-8763) = `elements[k] + (Heat: Σmods(heat_scaledByStrength)×strength + cust(heatAdd)) + (Electricity: Σmods(electricity_scaledByStrength)×strength)`, which feeds DoT tick damage (`getTickStatusDmg`, L9179-9190), not the hit.

Only numbers are summed: nested objects (`element`, `phys`, `SMITE`, `base_noncrit`, `flat_critmult_below_cchance`, ...) are read by dedicated functions below.

`findOneModsStat(modsPanel, key)` (L8798-8809) returns the **first** mod's raw `actions[key]` object (cached), or `[]`.

**Riven construction** (`showRiven`, L7273-7340): riven stat rows are iterated **in reverse UI order** (`.get().reverse()`, L7279). Value conversion: `comboDuration/initialCombo/range/punch_through` raw; faction rows (`Corpus/Grineer/Infested/Techrot/Scaldra`) `v − 1` (input is a multiplier such as 1.45); everything else `v/100`. Rows with `addt` go into `action[addt][type]` (`element`, `phys`, `SMITE`); "Grineer" also writes `Kuva Grineer`, "Infested" also `Infested Deimos`. Riven `speed` also writes `charge_time = v` and `FOR_WEAPON["primary-bow"] = {speed: 2v, charge_time: 2v}`. Riven stat catalogue: L14690-14740 (`base`, `multishot`, `crit_chance`, `crit_mult`, `status_chance`, `status_damage`, `speed`, the four primaries as `element`, the physicals as `phys`, `addViral/addBlast/addGas/addMagnetic/addRadiation/addCorrosive`, faction `SMITE`, `crit_chance_weakp` (also under `SMITE`, see quirk Q14), `mult_for_head` "Weak Point Damage", `magazineSize`, `reloadTime`, `ammoCapacity`, `ammoEff`, `shot_speed`, `comboDuration`, `initialCombo`, `melee_combo_eff`, `range`, `punch_through`, `zoom`, `recoil`, `status_duration`). At most 3 primaries may be chosen (L14760-14765).

---

## 2. Base damage multiplier (`findAllBaseModsDmg`, `BaseModsDmg`, `getNonCritModProps`, `getModdedBaseDmg`)

`findAllBaseModsDmg(attack)` (L8430-8461) returns a list of additive "base damage" bonuses:
- melee and attack.type ∈ {`s`,`hs`} (slam / heavy slam): `Σmods(slam_base)` if > 0 (L8433-8438);
- melee and heavy (`isNeedCalcHeavy(attack, is_Heavy)`, L13253-13259: attack.isHeavy wins if present, else the global Heavy option): `Σmods(base_heavy)` if > 0 (L8439-8441);
- weapon `unique.base` if > 0 (L8442-8443); `cust(base)` **if > 0** (L8444-8445); `getBasePerMagasize()` if > 0 (L8446-8447) = `round2(0.33 × √(modded magazineSize))` when any mod has `base_per_magasize` (Primary Plated Round, L4648; formula L10235-10239);
- every mod slot: `(isMaxCond && WITH_COND.base ? WITH_COND.base : actions.base) × setMult` (any sign) (L8448-8459).

`BaseModsDmg(nonCrit, attack)` (L8462-8467):
```
S = Σ findAllBaseModsDmg(attack)
nonCrit == false: BM  = 1 + S + (weapon.strengthType == "mesa" ? 1.5 × strengthMult : 0)
nonCrit == true : BMn = (1 + S + NC.ADD) × (1 + NC.MULT)        # no Mesa term
NC = getNonCritModProps()  (L8468-8479): Σ over mods of actions.base_noncrit.value and actions.base_uncrit_unstatus.value,
     bucketed by their .type ("ADD" or "MULT"); e.g. Phenmor "Devouring Attrition" {value:20, chance:0.5, type:"MULT"}.
```
So **all base-damage sources are additive with each other** (Serration-type, riven damage, heavy/slam base, unique base,
custom base, Plated Round); Mesa strength adds into the same bucket; the non-crit Incarnon perks are applied as an extra
ADD term and/or a separate multiplier only in the "non-crit" path (`reCalcWithNonCritBase`, §7.3).

`getModdedBaseDmg(attack, nonCrit)` (L8186-8193) = `Σ attack.damage × BaseModsDmg(nonCrit, attack)` — used only as the
status-proc base (`kb`, `fa`, L10393-10394).

---

## 3. Base damage before elements: Incarnon flat damage, `damage_mult`, progenitor element, swaps

### 3.1 Flat "+X damage" (`addDmgToBase`, L13193-13205)
```
for each attack p, each type q of currWeaponInitial.attacks[p].damage:
   T0   = Σ currInitialAttacks[p].damage           # listed base, WITHOUT progenitor
   r    = ((T0 + flat) × damage_mult × 100) / T0 / 100        # damage_mult = weapon.damage_mult ?? 1
   dmg[q] = (currWeaponInitialAfterElementChange[p].damage[q] ?? currInitialAttacks[p].damage[q]) × r
```
i.e. the flat bonus is spread **proportionally to the existing type split** (including a progenitor element when present,
because `currWeaponInitialAfterElementChange` holds it), and the ratio uses the listed total without the progenitor.
Sources: Incarnon perks `flat_change_dmg` (e.g. Furis "+28", sample JSON), custom stat "z" (`flat_change_dmg`), weapon
`unique.flat_change_dmg_by_strength × strengthMult` (L9640-9643).
Because `currWeaponInitial` is rewritten, `T(atk)` (= `getTotalBaseDmg`) **includes** the flat bonus everywhere downstream
(elemental % base, quantization scale, status base).

### 3.2 Progenitor element (`addPElement`, L12885-12901)
UI: Tenet / Kuva Lich / Coda tags show a dropdown {toxin, cold, heat, magnetic, electricity, radiation, impact} and a
value 25-60 (default 60) (L7940-7966; handlers L14625-14655).
```
for each attack g: add = getTotalBaseDmg(g)_before × val      # val = pct/100, computed from currInitialAttacks reset
   damage[elem] += add   (or create the key; new keys are appended LAST in JS key order)
   same for currWeaponInitialAfterElementChange[g]
```
For these tags `reCalc` does not reset `currWeaponInitial.attacks` (L9649-9657), so the element persists and becomes an
**innate element** for the combination logic (§4) and part of `T(atk)` (so elemental mods scale off base incl. progenitor,
consistent with WCF §3.2 "applies as weapon base damage"). Magnetic/Radiation progenitors are innate combined elements;
Impact progenitor is a physical type (affected by Impact mods via `phys`).

### 3.3 Swaps / unique conversions
- `swapDmgType("impact","puncture")` (L7342-7350) when `Σmods(impactToPuncture) > 0`: renames the key; **overwrites** an existing Puncture value (quirk Q3).
- `setChromaticBlade` / `setReactiveStorm` (L7351-7396): attack 0 becomes a single element/type equal to the total base; mod `status_chance` rescaled by strength.
- `addWeaknessDmgType` (L7469-7495): any mod whose catalogue entry has `add_weakness_dmg` (e.g. Sentient Incision, L4899-4903) gets its actions **replaced** by a faction-dependent "add" key: Infested → `addHeatNotCombined`, Anarchs → `addElectricityNotCombined`, Narmer → `addToxinNotCombined`, Grineer/Kuva Grineer/Zariman/Scaldra → `addCorrosive`, Corpus/Corpus Amalgam/Techrot → `addMagnetic`, Orokin → `addViral`, Sentient/Murmur → `addRadiation`, Infested Deimos → `addGas`, anything else → Grineer's (`addCorrosive`).

---

## 4. Elements and element combination (`newElements` L12902-13069 + helpers)

### 4.1 Inputs
- `p = getWeaponElement(baseElements)`, `q = getWeaponElement(comboElements)` (L8503-8513): per attack name, the innate damage keys that are primaries (Heat/Electricity/Toxin/Cold) or secondaries (Blast/Corrosive/Gas/Magnetic/Radiation/Viral). Includes the progenitor element (§3.2). Match is `elementName.toLowerCase().includes(key)` (substring test).
- `k = getListElementMods()` (L8606-8629): ordered list of mod elements = keys of `actions.element` (top-level only, **not** `WITH_COND.element`) in **slot order 0..29**, first occurrence wins; then appended (if > 0 and not yet listed) in fixed order: `toxin` (add_toxin + uniq + add_toxinS×strength), `electricity` (addElectricity + uniq + addElectricityS×strength), `cold` (addCold + uniq **without** addColdS), `heat` (addHeat + uniq + addHeatS×strength).
- `t = getSummElementsFromMods()` (L8549-8605): value per primary, **not set-scaled**:
  - per slot: if `isMaxCond` and the mod has `WITH_COND`: use **only** `WITH_COND.element` (nothing if it has none); otherwise use `actions.element` (L8555-8582);
  - plus the `add_*` sources above (L8583-8602, addCold here *includes* addColdS×strength);
  - Violet archon shard: `Electricity += (#crimson + #azure + #violet shards) × Σ electricityBonus of violet shards carrying Electricity` (L8556-8603; data L5385-5401: +30 % and +10 %/eligible shard, tauforged +45 %/+15 %). Counting includes the violet shard itself.
- `h = lichElement.val > 0 && lichElement.elem != ""` (L12910).

### 4.2 Pairing algorithm (per attack `w`, L12911-12940)
```
list = [ucFirst(e) for e in modElementList]            # slot order
     + [ucFirst(e) for e in innatePrimaries(w)]        # in the attack's damage-key order (progenitor last)
     + [ucFirst(e) for e in innateSecondaries(w)]
list = dedupe_keep_first(list)                          # removeSecondWeaponElement L8646-8653
if len(list) odd: list.append("")
E = {}; G = {}                                          # E: result fractions, G: mod sums that formed a secondary
for i in 0,2,4,...:
    a, b = list[i], list[i+1]
    H = t[a] + t[b]                                     # mod % only; innate elements contribute 0
    c = getTypeComboDmg(a, b)                           # L8519-8548
    if c != "":
        if c is secondary: G[c] += H
        E[c] += H
    if a is secondary: E[a] = 0                         # innate secondary: mark "keep innate"
    if b is secondary: E[b] = 0
```
`getTypeComboDmg(a,b)`: same → a; one empty → the other; primary + secondary → **the primary** (the secondary is not
broken and not merged); Heat+Cold=Blast, Electricity+Toxin=Corrosive, Heat+Toxin=Gas, Cold+Electricity=Magnetic,
Heat+Electricity=Radiation, Cold+Toxin=Viral; anything else (e.g. two secondaries) → "".

Consequences:
- Mod primaries pair strictly two-by-two in slot order (slot 0 with the next element, etc.), i.e. "each new primary combines with the next uncombined primary" (WCF §5.2).
- A mod element equal to an innate primary is deduplicated into the **mod's** position, so the innate element is "pulled forward" (WCF §5.2 rule 7) and its innate value is added to whatever it ends up in (via `B`, below).
- Innate primaries are placed **after all mod elements** (and after slots 13-29: warframe mods, aura, Incarnon perks, shards), in the attack JSON's key order, progenitor last — not in HCET order (quirk Q6).
- Innate secondaries come last; pairing a leftover primary with an innate secondary leaves both separate.

### 4.3 Writing the damage table (L12941-12973)
`currWeapon.attacks = clone(currWeaponInitial.attacks)`; delete all innate primary and secondary keys; then for each key
`w` in `E` (Tb = `getCurrAttackBaseDmg(currWeaponInitial.attacks[d])` = `T(atk)`, L8641-8645):
```
if E[w] == 0:
    if w is an innate primary:      dmg[w] = innatePrimary[w]                       # untouched innate
    elif w is an innate secondary:  dmg[w] = innateSec[w] + (G[w] > 0 ? G[w]×Tb + (w's components include
                                              the progenitor element ? lichVal × T0 : 0) : 0)
    else:                           dmg[w] = Σ innatePrimary[c] for c in components(w)   # two innate primaries combined
else:
    dmg[w] = Tb × E[w] + Σ innatePrimary[x] for x in innate primaries that are components of w or equal to w
```
`getLichDmgPart` (L12879-12884) = `lichElement.val × Σ currInitialAttacks[d].damage`.

The **innate-secondary + same secondary formed by mods** branch is the 2025.04.28 fix ("damage values were calculated
incorrectly when combining elements, in cases where a weapon contained a base combined element and mods included elements
required to create the same base combined element ... the element combination itself was determined correctly, but the
values were calculated incorrectly", index.txt L273-276): the innate secondary's key is forced to 0 in `E`, so the
mod-made amount is carried separately in `G` and added back. The progenitor term was added by the 2025.08.14 fix
("incorrect damage bonus values ... with combined elements and Lich elements", index.txt L245) and re-adds a progenitor
primary that was consumed into that same secondary. Result: mod-made secondary of the same type merges with the innate
one (same as WCF §5.2 example for Tenet Detron, which is itself marked UNVERIFIED there).

### 4.4 Non-combining element additions (L12974-13067), applied after combination
With `Tb = getTotalBaseDmg(d)` (base incl. flat and progenitor):
- `dmg.heat += Tb × (aUniq(addHeatNotCombined) + Σmods(addHeatNotCombined) + (Σmods + uniq)(addHeatNotCombinedScaledByStrength) × strength + cust(heatAdd))` — Fireball Frenzy, weakness mods vs Infested, custom "heatAdd";
- same for Electricity (Shock Trooper, weakness vs Anarchs), Cold (Freeze Force), Toxin (weakness vs Narmer) — no custom term;
- `dmg.viral += Tb × (Σmods(addViral) + uniq(addViral) + nourishBuff.val/100)`; `dmg.corrosive/radiation/magnetic/gas/blast += Tb × (Σmods(addX) + uniq(addX) + Σmods(addXS) × strength)` (riven secondary-element rows, weakness mods, Damzav-Vati, Nourish).
These create a new key if absent (which also increases the Condition-Overload status count, §6).

### 4.5 Worked examples (hand-traced)
- **Furis, Normal Attack** (I 3 / S 3 / P 14, T = 20), Primed Heated Charge (`element.Heat 1.65`, L422): list `[Heat]` → `E.heat = 1.65` → heat = 20 × 1.65 = 33.
- **Furis, Incarnon Form** (Heat 100) + same mod: list `[Heat, Heat]` → dedupe `[Heat]` → heat = 100 × 1.65 + 100 = 265.
- **Kuva Nukor** (Radiation 21), progenitor Heat 60 % (+12.6 Heat, T = 33.6), a 90 % Electricity mod: list `[Electricity, Heat, Radiation, ""]` → (Electricity, Heat) = Radiation, H = 0.9, G.rad = 0.9; (Radiation, "") → E.rad forced 0 → radiation = 21 + 0.9 × 33.6 + 0.6 × 21 = 63.84; Heat key removed.

---

## 5. Physical-type mods (`getPhysMods`, L8630-8640)
`phys[Impact|Slash|Puncture] = Σ over mods of actions.phys[type] × (1 + uniq(physModsInc))`. Read from `actions.phys`
directly (not `WITH_COND`, not set-scaled). Applied as a **separate multiplier** `(1 + phys[type])` on that type only
(§6), after base mods/CO — matches game (WCF §2). Adding Impact to a weapon without Impact has no effect (key absent).

---

## 6. Per-type damage after base mods, CO and physical mods (`reCalcWithTypeBase`, first loop L9372-9418)

Constants (L9356-9371): `cyte = (strengthType == "cyte") ? 1.25 × strengthMult : 1`; `flat` as in §0;
`SS = 1 + Σmods(multSharpshot)` (Longbow Sharpshot `WITH_COND 3`, L4657); `cold = Σmods(base_per_cold)` (Secondary
Shiver `WITH_COND 4.5`, L4556); `FIN = 1 + Σmods(finisherDmg)`; immuneAll = enemy immune to "all".

Per attack F:
```
BM  = BaseModsDmg(false, attack)                                   (§2)
N   = getCountDifferentStatuses(attack)                            L12015-12020
    = manualCountOfStatuses.is_set ? manual value
      : (#keys in the attack's modded damage table) + (externalVirus.is_set && no viral key ? 1 : 0)
CO  = Σmods(base_per_status) + cust(base_per_status) + aUniq(base_per_status)
isSlam = attack.name contains "slam" (case-insensitive);  isAoE = attack.shot_type == "AoE"
mul =  if !isMaxCond or isAoE or isSlam:                  BM × SS
       elif (CO > 0 or cold > 0) and !immuneAll:
            if attack.isCoMult or option is_MultiplicativeBasePerStatus:
                 (1 + cold + N × CO) × BM × SS                      # multiplicative CO
            else:
                 N × CO + cold + BM × SS                           # additive CO
       else: BM × SS
dmg[type] = dmg_after_elements[type] × mul × (1 + phys[Type]) × (type == "finisher" ? FIN : 1)

if flat > 0 and !attack.isCoMult and !is_MultiplicativeBasePerStatus:           # Incarnon-flat override, L9402-9414
    z = flat / Q                     # Q = number of damage keys of this attack (after elements)
    dmg[type] = z × BM × (1+phys)  +  (dmg_after_elements[type] − z) × (N × CO + cold + BM) × (1+phys)
dmg[type] = max(dmg[type], 0)
```
Note `dmg_after_elements` already contains the flat bonus (proportionally, §3.1) and mod elements computed from `T`.

### 6.1 Where "damage per status" (Condition Overload family) goes — complete list
Keys: `base_per_status` (Condition Overload 0.8 L172, Galvanized Aptitude/Savvy 0.8 L107/L123, Galvanized Shot 1.2 L115
— Galvanized values are pre-multiplied to max stacks; custom stat "b"; attack `unique.base_per_status`) and
`base_per_cold` (Secondary Shiver, always max). In the changelog Secondary Shiver "moved to the Condition Overload damage
type category" (index.txt L307-308).

| # | Place | Lines | CO treatment |
|---|---|---|---|
| 1 | Direct damage, normal path | L9389-9401 | additive `N·CO + cold + BM·SS` or multiplicative `(1+cold+N·CO)·BM·SS` |
| 2 | Direct damage, Incarnon-flat path | L9402-9414 | additive only; flat share `z` gets `BM` only (no CO, no Sharpshot, no finisher); **no isMaxCond / AoE / slam / immune-all check** |
| 3 | Non-crit (Incarnon "non-crit" perks) direct damage | `reCalcWithNonCritBase` L9218-9246 | same as #1 but with `BMn` and **without** Sharpshot / finisher / flat split |
| 4 | Status-proc base `kb` (crit path) | `getModelQueueDmg` L10827-10846 | `Σ_type base × (1+cc+fc·Ka)·Mc·tc` (mult.) or `Σ base × (fc·Ka + cc + Mc·tc)` (add.); flat split `sd·Mc + (ud−sd)(fc·Ka+cc+Mc·tc)` in additive mode; then `× faction × dbl_mult` |
| 5 | Status-proc base `fa` (non-crit path) | L10847-10851 | same with `Uc = BMn`, no Sharpshot, no flat split |

(In #4/#5 `Ka` = CO, `cc` = cold, `fc` = N, `Mc` = BM, `Uc` = BMn, `tc` = SS, `sd` = flat, `ud` = Σ initial base incl. flat;
L10395-10404, L10521, L10560-10566. Base = `currWeaponInitial.attacks[q].damage`, i.e. **pre-element** types; elements are
not re-applied to the status base.)

What decides additive vs multiplicative:
- per attack flag `isCoMult` in the weapon JSON (e.g. Catabolyst "Partial Reload Impact", samples/weapon-Catabolyst.json), **or**
- the global option "Property 'damage per status' calculated as multiplicative (~Condition Overload)" (`optMultiplicativeBasePerStatus`, L12103-12105; index.txt L77; URL key `o.md`, L9708).
- Weapon tags / compTags / categories / trigger type play **no** role (checked: no reference to tags in these branches).

What switches CO off: `isMaxCond == false`; attack `shot_type == "AoE"` (changelog: "Galvanized Savvy do not take into
account for all AOE type attacks", index.txt L165); attack name containing "slam" (changelog 2025.01.24 "Disabled ...
Condition Overload mod ... for Slam Attacks", index.txt L309-310); enemy immune to "all"; CO and cold both 0. Except
in path #2, where only "flat > 0 and additive" matters (quirk Q2). The optimizer also drops `base_per_status` mods from
the pool for AoE attacks (L16818, L16914).

Status count `N` is **static**: the number of damage types on the (modded) attack, not the statuses actually on the
enemy (quirk Q4). Option "count of statuses on enemy" (`manualCountOfStatuses`, L12096-12098) overrides it.

The "hidden bug simulation" of 2025.03.12 ("Incarnon perk 'increase Damage by +x' when used with mods like 'GunCO'
does not fully work with them on some attacks", index.txt L292-293) is path #2/#4's flat split: the flat portion gets
base-damage mods but not CO. This matches the wiki statement that the additive-CO recalculation omits Incarnon flat base
damage (WCF §11).

---

## 7. Per-layer per-type maps, faction, quantization (`reCalcWithTypeBase` L9488-9628)

Constants (L9492-9498 + L9366-9370): `FAC = getFactionMod()`, `MIR = mirage ? 1 + val/100 : 1` (Mirage buff option),
`MUL = 1 + Σmods(multiple)` (Double Tap, Power Throw, Synth Charge, Secondary Surge... one shared additive bucket),
`FRZ = 1 + Σmods(dmgMultAgainstFrozen)`, `DKY = 1 + Σmods(multDAIKYUBroadhead)` (Spring-Loaded Broadhead),
`DBL = 1 + Σmods(dbl_mult)` (e.g. Mecha Empowered aura `1.5`, L4890), `MAD = optMadurai && !compTags POWER_WEAPON ? 1.3 : 1`
(physical types only), `cyte` (§6).

```
unit(F)       = T(F) × BaseModsDmg(false, F) × FAC
value(F,type) = dmg[type] × FAC × MIR × MUL × FRZ × DKY × DBL × (type is physical ? MAD : 1) × cyte
q             = setQuantize(unit, value)              # L9787-9791: s = unit/16; value == 0 ? 0 : round(value/s)×s
health[F][type] = calcPercentAdd(q, typeOfFaction[enemy.faction][Type])       # L8182-8185, table L5503-5518
```
- `calcPercentAdd(b, d) = d > 1 ? b·d : (d ? b − b·d : b)` — a 1.5 weakness multiplies by 1.5; a 0.5 resistance gives `b − 0.5b`.
- Layers: `health` (if the attack has Toxin **and** the enemy has shields: only the Toxin entry), `healthFull` (all types; used once shields are down), `shield` (if Toxin present: all but Toxin; computed only when enemy has shields), `overguard` (no faction-type modifier; Void × 1.5), `armor`/`armorFull` = `armorDmgWithDecreaseArmor(1, health/healthFull)` (armor stage is another agent's topic). L9506-9627.
- Because FAC multiplies both unit and value, quantization is effectively `round(16 × dmg × MIR×MUL×FRZ×DKY×DBL×MAD×cyte / (T × BM)) × T × BM × FAC / 16`: faction and base mods are outside the rounding; CO, Sharpshot, physical mods, finisher, elements and the MIR..cyte multipliers are **inside** it.
- `T` here is the base incl. flat and progenitor but excluding mod elements; the unit uses `BM` (no CO, no Sharpshot).

### 7.1 Faction multiplier (`findAllFactionMods` L8480-8502, `getFactionMod` L9104-9109)
```
fac[f] = 1 + Σ over mods (isMaxCond ? (WITH_COND.SMITE || SMITE) : SMITE)[f] × setMult
         for f in {Grineer, Sentient, Corpus, Orokin, Infested, Stalker, Kuva Grineer, Infested Deimos, Techrot, Scaldra}
b      = fac[enemy.faction] (falsy → 1);  roar = rhinoBuff.is_set ? rhinoBuff.val/100 : 0
FAC    = b != 1 ? b + roar : (roar > 0 ? 1 + roar : 1)
```
Roar is additive with faction mods (changelog 2024.05.08, index.txt L362). Faction is applied once to direct damage and
**twice** to DoT (`Sa = 0.35 × kb × oa` with `kb` already × `oa`, L10393 / L10915), matching game.

### 7.2 Worked quantization example (Furis vs Corrupted Heavy Gunner, faction Orokin; test case research/test-cases/user-example-furis.json)
Normal Attack, Hornet Strike `base 2.2` (L42) → BM = 3.2; no CO; unit = 20 × 3.2 / 16 = 4.
Impact 9.6 → round(2.4)=2 → 8; Slash 8; Puncture 44.8 → 11 → 44 → ×1.5 (Orokin Puncture) = 66; Heat 105.6 → 26 → 104.
Health-layer sum 186 vs unquantized 192.0 (−3.1 %). Incarnon Form: 265 × 3.2 = 848, unit 20 → 840.

### 7.3 Non-crit maps (`reCalcWithNonCritBase`, L9212-9349)
Called from `getModelQueueDmg` only if a mod has `base_noncrit` / `base_uncrit_unstatus` (L10342-10346). Starts from
`currWeaponAttacksWithElements` (snapshot of the element table, L9353-9355) and uses `BMn` in place of `BM` both in the
damage and in the quantization unit; no Sharpshot, no finisher, no flat split, no `cyte`; otherwise identical layer logic.

### 7.4 "ifMaxStat" uniques (L9419-9469)
If an attack's `unique.ifMaxStatImpact/Puncture/Slash` exists and that physical type is the largest after §6
(`findMaxPhysStat`, L8833-8842), it re-computes: Impact → fire rate with extra `speed`; Puncture → range + `range`;
Slash → status chance with an extra additive `status_chance` (stored in global `globalModStatusChance`, quirk Q16).

---

## 8. Crit chance (`getFinalWeaponCritChance`, L8194-8267)

Signature `(atk, i, extraFlat=0, overrides=[], isHeavy, comboMult, isHeavySlam, statusCount)`. Display call:
`setAdditionalMods` L8857-8866 (no overrides, extraFlat 0, statusCount 0). Per-hit call: `getModelQueueDmg` L10648-10669
with overrides `Ld` (L10523-10541), `extraFlat = 0.05 × punctureStacks (if not AoE/immune) + Wd` and `statusCount = fc`.

```
WPcc   = headshotOption ? Σmods + cust + uniq (crit_chance_weakp) : 0
WPmult = (WPcc > 0 and (is_MultiplicativeWeakCC or atk.isWPCritMult)) ? 1 + WPcc : 1;  if WPmult != 1: WPcc = 0
HCC    = headshotOption ? uniq(headCritChance) + Σmods(headCritChance) : 0
SCC    = from first mod's inc_add_crit_chance_per_status_up {val,max}: min(statusChance%·val, 100·max)/100  (uses modded SC)
combo  = (comboMult > 0 and (heavySlam or heavy)) ? comboMult − 1
         : (combo setting on and > 0 ? setting − 1 : combo_counter)        # combo_counter default 11 or weapon.maxHeavyCombo
BR     = combo × Σmods(crit_chance_per_combo)                              # Blood Rush / Gladiator set
X2     = heavy ? Σ getX2CritChanceForHeavy() : 0                           # L8810-8832
flat%  = 100 × ( Σmods(flat_crit_chance) + cust + uniq(critFlatChancePerStatuses)·statusCount + uniq(flat_crit_chance) + extraFlat )
add%   = 100 × ( Σmods(add_crit_chance) + SCC )
mult   = 1 + Σmods(mult_crit_chance) + cust(mult_crit_chance)
rel    = 1 + Σmods(crit_chance) + uniq(crit_chance) + zoom(crit_chance) + cust(crit_chance) + WPcc + aUniq(crit_chance) + X2 + BR + HCC
CC%    = ((atk.crit_chance + add%) × rel × WPmult + flat% + optFlatCrit.val) × mult
```
- Relative bonuses (Point Strike, riven, Blood Rush, additive WP CC, head CC, zoom, attack unique, heavy doubling) are one additive bucket; `add_crit_chance` adds to the **base** before it; `flat_crit_chance` (Arcane Avenger 0.45, L5158; Incarnon perks) and the "flat crit" option (raw % units) add after; `mult_crit_chance` multiplies everything last.
- `getX2CritChanceForHeavy`: Sacrificial-set `crit_chance × set mult`, plus raw top-level `crit_chance` of mods named Riven / Galvanized Steel / True Steel / Amalgam Ripkas True Steel; plus `cust(crit_chance)` once **per non-empty slot** (quirk Q8).
- Side effect: sets `combo_counter = weapon.maxHeavyCombo` if present (L8264-8265).
- `setAdditionalMods` computes CC, then SC (which may use CC), then recomputes CC if `inc_add_crit_chance_per_status_up` exists (L8857-8918).

## 9. Crit multiplier

### 9.1 Display value (`setAdditionalMods` L8868-8892)
```
fcm  = inc(flat_critmult) + (CC_modded < cond ? val : 0)      # flat_critmult_below_cchance {val,cond}, last mod wins (reduce)
CM   = ((atk.crit_mult + fcm) × (1 + Σmods(crit_mult) + cust + aUniq(crit_mult))
        + (melee & heavy ? Σmods(heavy_crit_mult) : 0)
        + aUniq(crit_mult_add) + Σmods(crit_mult_add) + cust(crit_mult_add)) × (1 + cust(crit_mult_mult))
```
### 9.2 Per-hit value (`getFinalWeaponCritMulti` L8268-8329, via `setCritDmgToHit` L9822-9852)
```
k    = inc(flat_critmult) + (ccArg < cond ? val : 0)
rel  = 1 + Σmods(crit_mult) + cust(crit_mult) + aUniq(crit_mult) + (melee & heavy ? Σmods(heavy_crit_mult) : 0)
abs  = Σmods(crit_mult_add) + aUniq(crit_mult_add) + cust(crit_mult_add)          # Arcane Crepuscular etc.
P    = Σmods(incr_CM_by_punc_status) > 0 ? min(puncShare × atk.status_chance × 0.1, 50) : 0
xm   = 1 + cust(crit_mult_mult)
CMm  = (atk.crit_mult + k) × rel
if ccArg <= 0: return 1
if doubleCrit: ccArg += 100                                  # double_crit (Vigilante etc.) raises the tier by one
tier = ccArg <= 100 ? 1 : ceil(ccArg/100)
hs   = atk.no_headshot_mult ? 1 : headShotMult
non-headshot: (1 + tier × (CMm − 1) + abs + P) × xm
headshot:     (hs × (1 + tier × (2 × CMm − 1)) + abs + P) × xm          # crit-headshot = 2× CM, times weak-point part
```
Tier selection (`setCritDmgToHit`): for CC ≥ 100 % the roll is on `frac(CC)`; if it fails `ccArg = CC − 99`, otherwise
`ccArg = CC`, so tier = floor(CC/100) or +1. Unique attack `crit_mult` (Rubico zoom 0.35 / 0.5, samples) is in `rel`.
Base CM is **not** quantized to the 32/4095 grid (WCF §9).

### 9.3 Headshot terms (setAdditionalMods L9070-9084)
`headShotMult = 3 + Σmods(headshot_mult) + cust + zoom + uniq` — `headshot_mult` is the weak-point-damage term stored
pre-multiplied by 1.5 (Primary/Pistol Acuity `5.25` = 1.5 × 3.5, L77/L89).
`headshotIncreaseDmg = (1 + Σmods(mult_for_head) + cust + zoom + uniq) × (1 + Σmods(mult_for_head_mult) + uniq)` —
Deadhead-type headshot bonus (Daikyu Target Acquired 0.75 L1117, Incarnon "Lingering Judgement" 0.5).
Per hit: `ta = headshotIncreaseDmg` always on headshots; `Ma = headShotMult` only on non-crit hits (crits carry `hs` in the
crit multiplier) (L10358-10368, L10777). This is `part × head × crit` as in WCF §10.

## 10. Status chance (`getFinalStatusChance`, L9888-9977)
Signature `(i, combo, extraRel, extraAbs, tauStacks=0, override=0)`; display call L8893-8901 passes
`tauStacks = statusDuration.Tau.max (10)` if the attack has Tau damage.
```
if override > 0: return override
if atk.noIncrStatus: return atk.status_chance
base   = Σmods(set_status_chance) > 0 ? that value : atk.status_chance          # percent units
SCfromCC = first mod's inc_flat_status_chance_per_crit_up {val,max}: min(CC_modded·val, 100·max)/100   (only when not "set")
flatSC = 100 × ( inc(flat_status_chance) + SCfromCC + aUniq(flat_status_chance) )
comboS = (combo > 0 and (heavySlam or heavy) ? combo−1 : setting−1 or combo_counter) × Σmods(status_chance_by_combo)
rel    = 1 + Σmods(status_chance) + extraRel + comboS + cust(status_chance) + (heavy ? Σmods(h_status_chance) : 0) + aUniq(status_chance)
abs    = Σmods(absolute_status_chance) + extraAbs + cust(absolute_status_chance) + aUniq(absolute_status_chance)   # percent units
mult   = 1 + Σmods(status_chance_mult) + cust(status_chance_mult) + 0.1 × tauStacks
SC%    = ((base + flatSC) × rel + abs) × mult
```
Incarnon flat status (`flat_status_chance`, e.g. Furis "Elemental Balance" 0.28) is added to the **base** before mods
(via `inc()`), while `absolute_status_chance` (Phenmor perks `20`) is added after.

## 11. Other stats (`setAdditionalMods`, L8843-9094)
| Stat | Formula | Lines |
|---|---|---|
| Fire rate / attack speed | `base × (1 + Σmods(speed) + cust + zoom + aUniq(speed)) × (1 + Σmods(speed_mult) + aUniq(speed_mult))`, clamp ≥ 0; `set_speed_to_default` (mods + aUniq) → base | L8919-8940 |
| Burst delay | `base / (1 + speed bucket) × (1 + speed_mult bucket)` | L8941-8952 |
| Shot speed / radius | `base × (1 + Σmods(k) + aUniq(k))` | L8953-8966 |
| Punch through | `base + Σmods + aUniq` | L8967-8972 |
| Charge time | `base / (1 + Σmods(charge_time) + aUniq)` (riven fire rate also writes charge_time; ×2 on bows) | L8973-8982 |
| Multishot (attack and weapon) | `(base + Σmods(flat_multishot)) × (1 + Σmods(multishot) + cust + uniq(multishot))`; any `set_mutishot_to_default` (Acuity) → base; no lower clamp | L8983-9006 |
| Magazine | `round((base + Σmods(baseMagazineSize)) × (1 + Σmods(magazineSize) + cust + uniq) + Σmods(add_magazineSize))`; skipped if `disableMagazChange` | L9007-9018 |
| Reload | `reloadRate = base / (1 + Σmods(reloadRate) + uniq)`; `reloadDelay = base / (1 + Σmods(reloadTime) + uniq)`; `reloadTime = (reloadRate & reloadDelay) ? delay/(1+rt+cust+uniq) + magazine/reloadRate : base/(1+rt+cust+uniq)` | L9019-9041 |
| Ammo max | `floor((set_ammoCapacity or base) × (1 + Σmods + uniq))` | L9042-9052 |
| Combo duration / range | additive `base + Σmods + uniq` | L9053-9069 |
| Wind-up | `base / (1 + Σmods + uniq)` | L9059-9063 |
| Status duration | `curr = parseInt(duration × (1 + Σmods(status_duration) + uniq))` (truncated to whole seconds) | L9085-9093 |
Beams (`isBeam`): multishot is not rolled per shot (`procMultishotArray` returns 1s, L10050-10062).

## 12. Per-hit multipliers outside the stat maps (`getModelQueueDmg`, for context)
`Jb = multishotTier × (1+cust dmgVulnerability) × ta × Ma × crit × viral × magnetic × stanceMult × heavyComboMult
× (1+slam_mult) × strength(simple/excalibur) × additionalBuff × first-shot (1+f_multiple) × last-shot (1+l_multiple)
× TenetPlinxMag × DKY × sniperCombo × (1+electricityShardAbilityDmg) × abilityCombo × overguardMult` (L10860-10880).
- **Sniper combo** `J = atk.sniperCombo ? (setting ? val : 1.5) : 1` (L10326-10330) — a plain multiplier, default 1.5.
- **Stealth**: `crit = getStealthDmgBonus(crit)` when the option is on (L10826): non-melee 1; melee non-crit **7**; melee crit `8 + (crit − 1)` (L8330-8336). Applied after the Cold crit bonus (L10822-10825).
- **Heavy attack**: base_heavy (§2), heavy_crit_mult (§9), h_status_chance (§10), `h_status_damage` in status damage (L10498-10503), x2 crit chance (§8), combo for crit/status from the per-hit `comboMult`, and `Ha` combo multiplier in `Jb`.
- **Madurai**: only in the per-type maps (§7), physical types only, not POWER_WEAPON; not in the status base `kb`.
- **Status-damage multiplier** `zb = (1 + Σmods(status_damage) + cust + heavy h_status_damage) × (1 + Σmods(multiple)) × (1 + dmgMultAgainstFrozen) × (1 + vuln_status_damage)` (L10498-10503).

---

## Suspected inspiration quirks
- **Q1** `reCalc` with flat = 0 calls `addDmgToBase(0)` (applies `damage_mult`) and then, for ordinary weapons, resets `currWeaponInitial.attacks` to `currInitialAttacks`, discarding `damage_mult`; with flat ≠ 0 it is kept (L9644-9657, L13194).
- **Q2** Incarnon-flat path (§6 #2) applies additive CO even when `isMaxCond` is off, on AoE attacks and on slams, and drops Sharpshot and finisher multipliers (L9402-9414).
- **Q3** `swapDmgType` overwrites an existing Puncture value instead of adding Impact into it (L7342-7350).
- **Q4** CO status count is the number of damage types on the weapon (+1 external Viral), not the statuses actually on the enemy; partial immunities ignored; zero-value keys, Tau/Void/True/Finisher keys count (L12015-12020).
- **Q5** Quantization grid 1/16 (L9787-9791); game is 1/32 since U40 (WCF §14). Several global multipliers (Mirage, `multiple`, frozen, Daikyu, `dbl_mult`, Madurai, Cyte) are inside the rounding.
- **Q6** Innate primaries (incl. progenitor) are ordered by JSON key order, progenitor last, not HCET (WCF §5.2 rule 5) (L12912-12917, L12885-12901).
- **Q7** `getFinalWeaponCritChance` in the per-hit path adds `cust(crit_chance)` twice (`Ld.crit_chance` already contains it, L10525-10528 + L8258).
- **Q8** `getX2CritChanceForHeavy` pushes `cust(crit_chance)` once per non-empty mod slot (L8828-8829 inside the slot loop).
- **Q9** Display crit multiplier adds `heavy_crit_mult` flat (L8878-8888) but per-hit adds it to the relative bucket (L8271-8292).
- **Q10** `crit_mult_add` and `incr_CM_by_punc_status` are added after tier scaling (`1 + t(CM−1) + abs`), not into CM (L8314-8328); game adds absolute CM into CDM before tiers (WCF §9).
- **Q11** `flat_critmult_below_cchance` compares against `ccArg`, which is `CC − 99` when CC ≥ 100 % and the extra tier fails (L9826, L8286); and `reduce` keeps only the last mod's value (L8280-8284).
- **Q12** Stealth melee non-crit multiplier is 7, but crits use `7 + crit`; wiki: 1 + 7 = 8 non-crit, additive with crit (Stealth.wiki L36-49). Weapon rank ignored.
- **Q13** Spring-Loaded Broadhead (`multDAIKYUBroadhead`) multiplies both the per-type map (L9369, L9513) and `Jb` (`va`, L10406, L10875): applied twice.
- **Q14** Riven "Weak Point Critical Chance" has `addt: "SMITE"` (L14722-14725), so it is stored as a faction entry and has no effect.
- **Q15** `findAllFactionMods` has no Murmur / Corpus Amalgam / Narmer / Anarchs / Zariman keys; Bane/Expel of the Murmur (`SMITE.Murmur`, L1493-1500) produces NaN → faction multiplier 1 (L8481-8492, L9106).
- **Q16** `globalModStatusChance` (ifMaxStatSlash) is never reset and is applied to attack index 0 only in proc rolls (L9459, L10001).
- **Q17** `getSummElementsFromMods` ignores a mod's top-level `element` when `isMaxCond` and the mod has a `WITH_COND` without `element` (Proton Snap duplicates it to work around this, L690); `getListElementMods` ignores `WITH_COND.element`; `addColdS` is counted in values but not in the list (L8621-8622 vs L8593-8597). Elements are not set-scaled.
- **Q18** A listed element whose summed value is 0 still occupies a pairing slot and leaves a 0-valued key (counted by CO).
- **Q19** `cust(base)` is ignored when negative; riven negative base is honoured (L8444-8445 vs L8458).
- **Q20** Burst delay is multiplied by `(1 + speed_mult)` instead of divided (L8942-8951).
- **Q21** Reload-speed mods (`reloadTime`) do not change `reloadRate` on per-round-reload weapons; only the `reloadRate` key does (L9019-9041).
- **Q22** Multishot has no floor at 1 for negative riven multishot (L8983-9006; WCF §7 says cannot go below 1).
- **Q23** Status duration truncated with `parseInt` (L9090-9092); Tau status assumed at max 10 stacks in the displayed status chance (L8898-8900).
- **Q24** Riven stat rows are read in reverse order, so two riven primaries enter the element list in reverse of their display order (L7279).
- **Q25** Riven "Weak Point Damage" maps to `mult_for_head` (head-bonus factor) rather than to the weak-point part term `headshot_mult` (L14728).
- **Q26** `getModelQueueDmg` status base `kb` includes `dbl_mult` only when `isMaxCond && !AoE && !slam` (L10845 vs L10393).
- **Q27** `incForm`/`normForm` take `WITH_COND` values even when `isMaxCond` is off (L8723-8724).
- **Q28** `setChromaticBlade`/`setReactiveStorm` assign both `currWeaponInitialAfterElementChange.attacks` and `currWeaponInitialAfterElementChange[0].damage` (inconsistent shape) (L7352-7362).
- **Q29** `calcPercentAdd` treats any modifier in (0,1] as "reduce by d" (`b − b·d`), correct only because the table contains only 0.5 and 1.5 (L8182-8185, L5503-5518).
- **Q30** Mod `multiple` sources share one additive bucket (L9366, L10465, L10498-10503) though the game treats such multipliers as independent (WCF §12).

## Open questions
- Where `damage_mult` is set on weapon JSONs and whether Q1 is intended (no sample contains it).
- Incarnon perk `type` field (e.g. Phenmor "Retribution's Vessel" `type: ["Normal Attack"]`) is not read by `showIncMod` (L7566-7579); is it applied elsewhere (optimizer L15658-15670)? Not traced.
- What `incr_CM_by_punc_status` represents in game (only its presence is used; formula `min(puncShare × SC × 0.1, 50)` added to CM, L8297-8311).
- Whether the heavy-attack doubling of riven / Galvanized Steel / True Steel / Sacrificial crit chance (L8810-8832) is game-accurate.
- Whether game counts Fireball Frenzy / Shock Trooper / Freeze Force / weakness elements as non-combining (`*NotCombined`), as the inspiration does.
- Whether Madurai's 30 % should also affect status procs (inspiration: direct physical damage only).
- Exact game rule for which attacks are `isCoMult`; the inspiration encodes it per attack in weapon JSON, so the full list requires the weapon data dump (data-pipeline agent).
- `addCold`, `addHeat`, `add_toxin`, `addElectricity`, `*S`, `impactToPuncture`, `flat_multishot`, `baseMagazineSize`, `set_status_chance`, `flat_critmult` keys do not appear in the static mod table (L2-5435); presumably in weapon `unique`/Incarnon JSON — not verified.
- Whether a progenitor primary absorbed into a mod-made secondary of the same type as an innate secondary (§4.3) can double-count when the progenitor is also listed through `p` — traced cases are correct, not exhaustively proven.
