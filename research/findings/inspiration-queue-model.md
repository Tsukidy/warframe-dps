Status: draft

# Inspiration per-hit simulation: `getModelQueueDmg` (min.pretty.js L10311-12014)

Source: `research/external/inspiration/min.pretty.js` (**L** + line numbers). Minified local names are kept in
`code font` so a reviewer can find them; every formula gives its line range. Sibling files referenced:
`research/findings/inspiration-stat-pipeline.md` (**SP §n**, how the per-type maps, crit chance/multiplier and status
chance are built) and cached wiki pages in `research/external/cache/wiki/` (**wiki:Page**).

Notation:
- `Σmods(k)` = `getStatValFromMods(k)`; `cust(k)` = `getCustomStat(k)` (L13262-13264); `uniq(k)` = `getStatFromUnique(k)`
  (weapon `unique` block, `WITH_COND` value if `isMaxCond`, L8782-8789); `aUniq(k)` = `getStatFromAttackUnique(k, atk)` (L8776-8781).
- `atk` = `currWeaponInitial.attacks[q]` (pre-element base table); `atkM` = `currWeapon.attacks[q]` (modded: per-type damage after
  base mods/CO/physical mods/elements written back at L9416, final status chance L8893, speed etc.).
- `FAC` = `getFactionMod()` = faction-mod multiplier with Roar added (L9104-9109, SP §7.1).
- `iDR[L]` = `enemyinnateDR[L]` (enemy JSON `innateDR`, L7712), `iDRs` = `currEnemy.innateDR.status`.
- `DR(a)` = `getDamageReduction(a)` = `sqrt(3a)/100` (L8148-8150; equals `0.9·sqrt(a/2700)`); armor value `A` = `currEnemyArmorVal`
  (already capped at 2700 and with the user armor-strip option applied, L8114-8116).
- `typeMod(T)` = `findTypeOfRes(T)` = faction damage-type modifier from `typeOfFaction` (L5503-5518, L9205-9211), `false` if none.
- `calcPercentAdd(b,d)` = `d>1 ? b·d : (d ? b−b·d : b)` (L8182-8185).
- `roll(chance%, tag)` = `F(chance, 100, hitId, tag)` = `memoRandom(hitId:tag:idx)·100 < chance` (L10313-10318).

---

## 1. Parameters, return value, side effects

```
getModelQueueDmg(b, d, g, p, q, k, t, h = [], w = p, x = p, m = {})          L10311
```
| Param | Meaning | Evidence |
|---|---|---|
| `b` | hit list ("full model") for one attack: `[{crit:1, statuses:[...], time, shotIndex?, multishotTier, melee_mult?, comboMult?, stanceStatuses?, hasITS?, ordinaryStatusCount?, hitId}]`. One entry per **pellet** (multishot expanded) except beams, whose multishot is folded into `multishotTier`. Deep-copied into `B` (L10312) so splices do not leak between layer passes. | built in `createDmgQueue` L12426-12546; `procMultishotArray` L10050-10062; `procStatusArray` L9978-10049 |
| `d` | status timeline: `d[i] = {status: {ticks}}` = stacks active on the enemy **at hit i, including hit i's own procs** (`timelineStatus` L10093-10136 adds `+1` at index `k` of the applying hit). Empty `{}` per hit when Sagek passive is active (built incrementally instead, L12546). | L12546, L12558 |
| `g` | direct damage of one hit on layer `p` before per-hit multipliers: `Σ_type currWeapon[w][atk][type] × (1 − elemRes[type])` (`getTotalTypeOfArmorDmg` L9110-9119). Callers pass `1` for armor; it is recomputed inside (§5). | L12554-12556, L12574-12578 |
| `p` | target layer: `"health"`, `"armor"`, `"shield"`, `"overguard"` | L12563, L12607, L12648, L12674 |
| `q` | attack index | — |
| `k` | `first_last_bullets` = `{first:{hitIdx:0}, last:{hitIdx:0}}` from `makeBulletsPositionMark` (only when Charged/Primed Chamber or Synth Charge equipped) | L12519-12524, L12182-12196 |
| `t` | per-attack status "masks" (100-slot weighted type arrays from `makeRNDMaskStatusArray`, L12239-12246) used for re-rolls | L12532 |
| `h` | shot schedule `{shotTimes, shotInfos, reloadInfos, totalReloads, totalShots}` (only copied into `arrInfo` for display/TTK) | L12490, L10590-10593 |
| `w` | key of the per-type damage map in `currWeapon` (`health`, `healthFull`, `armor`, `armorFull`, `shield`, `overguard`) | L12568, L12588, L12627 |
| `x` | key in `arrInfo` where per-hit results are written (same values as `w`) | L10572 |
| `m` | memo for deterministic random numbers, keyed `"hitId:tag:idx"`; the caller passes one object per attack (`t = Object.create(null)`, L12551) to all layer passes, so the **same crit/proc rolls are used on every layer** | L10313-10317, L12570 |

Return: `H = Σ_i totalDmg_i` (L12011-12013). **The caller ignores the return value** (all six calls L12559-12682 are
statements); results are consumed from `arrInfo[x][attackName][i]` by `getMedianFromTotalQueueDmg` (L12802-12805),
`getAvgQueueVal`/`getMedianQueueVal` (L12217-12238) and `calcTTKWithStatusUnified` (L12705-12715).

Call matrix (createDmgQueue L12550-12693): `health` always; `healthFull` if enemy has shields; `armor` if armor > 0;
`armorFull` if armor and shields; `shield` if shields; `overguard` if overguard.

Per-hit output record `arrInfo[x][y][i]` (L10588-10593, L10820, L10855-10859, L10952-11143, L11396-11997, L12010):
`timeM, shotInfo, reloadsInfo, statusOnEnemy (snapshot of d[i]), decrease_armor_mult (Fa), crit, statuses, meleeMult,
comboMult, status[type] = {d, nc, proc_d?, proc_nc?, instance_d?}, isNC, xataTotal, toxicLash, curr_dmg, toxinDmg,
debilitateDmg, debilitateHasCold, dmgOnStatusEff, dmgOnStatusEffToxin, otherDmg, totalDmg`.

## 2. Simulation frame (what the hit list represents)

- Simulated window `queueLength` seconds, default **20** (L5496), UI-adjustable (`#queueLen`, L14863-14864; optimizer 1-60, L16603).
- Guns: shot times from the scheduler `b()` in `createDmgQueue` (L12251-12406, §9); each shot expanded to `floor(MS)` pellets
  plus one with probability `frac(MS)` (`procMultishotArray`, rounding `frac` to whole percent, L10050-10062). Beams: one hit
  per tick, multishot becomes `multishotTier` = `floor(MS) + roll(frac)` and status chance × tier (L10002-10010).
- Melee: `N = round(queueLength × stanceHits / (stanceTotal/speed [+ windUp for heavy]))` hits evenly spaced at
  `i × queueLength/N` (L12436-12457, `getMeleeCountHits10s` L10137-10216); per-hit `melee_mult` and forced stance statuses
  cycle through the stance's `hits`/`statuses` arrays (L12115-12150); slam types `s`/`hs` use a single hit `[1]`.
- Status rolls on the base hits happen **before** `getModelQueueDmg` with plain `Math.random` (`procStatusArray`), then
  `setForceProcs` (L12845-12878) and `setProcsITS` (Internal Bleeding / magnetic-on-impact, L9853-9878).
- Crit is **rolled inside** `getModelQueueDmg` per hit (§3.3).

## 3. Per-hit loop (L10583-12012), in execution order

Constants computed once before the loop are listed with each use. Loop index `A` (= i), `cb = hitId` (L10584),
`ca` = copy of the hit's statuses (L10585), `Ia = melee_mult || 1` (L10586), `Ha = comboMult` if melee **heavy** else 1 (L10587).

### 3.1 Status rewrites before damage
1. **Sagek passive** (`fb = uniq("sagekPassive")` %, L10579): if the previous hit set `$b`, this hit's statuses are re-rolled
   with status chance forced to 500 % (`getFinalStatusChance(...,500)` returns 500, L9888-9889) and concatenated with the
   forced procs after `ordinaryStatusCount` (L10595-10607). Then **Secondary Encumber** in Sagek mode: if `wd = 100·Σmods(random_status_after_status) > 0`,
   the hit has statuses, and no Encumber proc happened at the same timestamp, `roll(wd)` adds one uniformly random type from
   Impact/Slash/Puncture + 4 base + 6 combined elements (L10608-10617). Timeline extended time-based (`extendTimeline(...,B)`,
   L10619). `$b` for the next hit = `crit > 1 && roll(fb)` (L10694). (Without Sagek, Encumber is applied in `procStatusArray` L9980-10040 / `setForceProcs` L12863-12876.)
2. **Primary Debilitate** (`vb`, L10447; mod L4616): collect combined elements with `≥10` stacks on the timeline **and**
   present in this hit; pick one at random, then one of its two components at random (`comboElementsInclude`, L5532-5543);
   extend the timeline index-based with that element and remember it in `Gc` (L10621-10639). Damage in §6.6.
3. **addPunctureStatus** (`Ab = Σmods`, key not in static mod table): adds `Ab` Puncture stacks to `d[i]` only (capped at 5),
   not propagated to later hits (L10640-10647).

### 3.2 Crit chance of the hit (L10648-10678)
```
punctureBonus = (Puncture on d[i] && !AoE && !immune(all|puncture)) ? 0.05 × punctureTicks : 0       L10648-10672
ic = getFinalWeaponCritChance(atk, q, punctureBonus + Ec, Ld, isHeavy, Ha, isHeavySlam, fc)          (percent)
```
`getFinalWeaponCritChance` (L8194-8267; SP §8):
`CC% = ((atk.crit_chance + 100·(add_crit_chance + incAddCCPerStatus)) × (1 + crit_chance_sum + zoom + cust + weakpointCC + aUniq + heavyX2 + combo×crit_chance_per_combo + headCritChance) × WPmult
        + 100·(flat_crit_chance + uniq + cust + critFlatChancePerStatuses×fc + punctureBonus + Ec) + optFlatCrit) × (1 + mult_crit_chance)`.
- `Ld` (L10523-10538) overrides the mod sums; `Ld.flat_crit_chance` includes `critFlatChancePerStatuses × fc` with
  `fc` = static status-type count (`getCountDifferentStatuses`, L12015-12020, 0 if immune to all).
- **Flat crit buff option** `optFlatCrit` ("optCrit", L12087-12089) is the `+ w` term: added after the multiplicative bucket,
  before `mult_crit_chance` (L8195, L8266).
- Combo for crit chance (Blood Rush `crit_chance_per_combo`): heavy attacks use the hit's `comboMult − 1`, others the
  combo setting − 1 or the global `combo_counter` (default 11) (L8218-8224).
- **Secondary Enervate** (`Wd = Σmods(incr_CC_by_hit_with_reset6tier2)` = 0.1, L4564): `Ec += Wd` after every hit
  (L10673), `Oc` counts hits with `tier ≥ 2 && crit > 1`; at 6, both reset to 0 (L10695-10696).
- **"crit_after_status"** (uniq, tag string `gotvaCritAfterStatus` suggests Gotva Prime): if the hit has ≥1 status and
  `roll(100·chance_inflict)`, `ic = 100·chance_crit` (overrides everything) (L10674-10678).

### 3.3 Crit roll and crit multiplier (L10679-10693)
```
critRoll   = roll(round(frac(ic/100) × 100), "crit")                                   L10679   (frac rounded to whole %)
doubleRoll = roll(100·(Σmods(double_crit) + uniq(double_crit)), "doubleCrit")          L10680-10683 (Vigilante set)
{val, tier} = setCritDmgToHit(atk, critRoll, ic, is_headShot, q, doubleRoll)          L10684-10691
crit_i = val;  if disableBodyCrit && !headshot: crit_i = 1                              L10692-10693
```
`setCritDmgToHit` (L9822-9852) + `getFinalWeaponCritMulti` (L8268-8329; SP §9.2):
- if `ic ≥ 100` and the fractional roll fails, the tier is computed from `ic − 99` (so tier = `floor(ic/100)`); if it
  succeeds, from `ic` (tier = `ceil`); if `ic < 100` and roll fails → `val = 1, tier = 0`.
- double crit adds +100 % to the chance used for the tier (= +1 tier) (L8312).
- tier `t`, `CM' = (atk.crit_mult + incFlatCM [+ flat_critmult_below_cchance]) × (1 + crit_mult mods + cust + aUniq + heavy_crit_mult)`:
  body: `val = (1 + t(CM' − 1) + crit_mult_add + puncCMbonus) × (1 + cust(crit_mult_mult))`;
  headshot (and attack not `no_headshot_mult`): `val = (headShotMult × (1 + t(2·CM' − 1)) + crit_mult_add + puncCMbonus) × (1 + cust)`.
  `headShotMult` default 3 (`headShotMultInitial`, L5456) + `headshot_mult` mods (L9070-9075).

### 3.4 Per-hit status side-effects that can add hits/statuses
- **Melee Duplicate** (`mDuplicate`, L10448; mod L4740 "On Base Critical Hits: 100 % chance to strike a second time"):
  if not a slam (`pd`: attack name contains "slam"), `crit > 1`, `tier === 1`, and the hit is not itself a duplicate:
  insert a clone right after it with id `hitId:meleeDuplicate` (own crit/double-crit roll, statuses re-rolled with the
  attack's final status chance + stance statuses), and insert a timeline entry = copy of `d[i]` plus the clone's statuses
  (+1 tick each, capped) (L10697-10741). The loop bound `B.length` grows, so the clone is processed as the next hit.
- **Secondary Cryogenic** (`rd = Σmods(onPunctureApplyCold)` = 3, L5190): if the hit has Puncture, enemy not immune to
  cold/puncture: push `rd` Cold statuses and extend the timeline time-based (L10742-10751).
- **Spiteful Defilement** (Phenmor Incarnon perk `flat_crit_with_status: 1`, samples/weapon-Phenmor.json): if `crit ≠ 1`,
  `Σ_types ticks on d[i] < 3` and (manual status count unset or `< 3`): `crit_i ×= (1 + nd)` (L10752-10760).
- **Hunter Munitions** (detected by mod name, `hasHunterMunitions` L9879-9881): if the hit has no Internal-Bleeding slash
  (`!hasITS`) and (`crit > 1` or CC ≥ 100 %): `roll(0.3 × min(ic,100))` → push `slash` (L10761-10766).

### 3.5 Per-hit multipliers (L10767-10852)
- **First/last shot**: `gb = 1 + Σmods(f_multiple)` if `k.first` has index i (Charged Chamber 0.4, Primed Chamber 1.0,
  L2163-2169); `ab = 1 + Σmods(l_multiple)` if `k.last` has i (Synth Charge 2, L2175) (L10775-10776). First/last are
  per magazine chunk of `H = floor(mag×ammoEffMult / ammoCost)` shots, all pellets of the first/last shot flagged
  (`makeBulletsPositionMark` L12182-12196); `l_multiple` only when `H > 6` (L12520-12521).
- **Headshot**: `Ca = headshot ? (atk.no_headshot_mult ? 1 : headShotMult) : 1` (L10358-10362),
  `ta = headshot ? (no_headshot_mult ? 1 : headshotIncreaseDmg) : 1` (L10363-10367; `headshotIncreaseDmg = (1+mult_for_head)(1+mult_for_head_mult)`, L9076-9084),
  `Ma = crit_i > 1 ? 1 : Ca` (L10777) — crits already carry `headShotMult` inside `val`.
- **Viral** (`Kd`: layer is not shield/overguard and enemy not immune to viral/all, L10476-10482):
  `I = 2 + 0.25 × (viralTicks − 1)` if Viral is on `d[i]` (L10780-10781). **External Viral** option (`optVirus`, stacks `s`):
  `ticks = min(d[i].viral.ticks + s, 10[, maxCountProc])`, or `s` if none on timeline; writes the ticks into `d[i]` (L10782-10798).
- **Magnetic** (`Vd`: layer is shield or overguard, not immune, L10483-10488): `Q = 2 + 0.25 × (magTicks − 1)` (L10802-10804).
- **Armor multiplier `Fa`** (armor layer only, L10805-10819), multiplicative, floored at 0:
  `Fa = (1 − decr_armor_by_punc × punctureTicks) × (1 − (0.2 + 0.06 × corrosiveTicks)) × (heat on d[i] || Napalm attack || reconifexHeat>0 ? 0.5 : 1)[if not heat-immune] × (1 − Σmods(armor))`.
  Effective armor = `A × Fa`.
- **Cold crit bonus** (not cold-immune, `crit > 1`): `crit_i += 0.05 × coldTicks`, and `+0.5` more at exactly 10 ticks (L10821-10825).
- **Stealth** option: `crit_i = getStealthDmgBonus(crit_i)` = melee only: `7` if non-crit else `crit_i + 7`; non-melee 1 (L10826, L8330-8336).
- **Status base `kb` (crit hits) / `fa` (non-crit-perk hits)** — recomputed per hit only when `isMaxCond && !AoE && !slam`
  (L10827-10852; SP §6.1 rows 4-5): `kb = Σ_type atk.damage[type] × COterm × FAC × (1 + Σmods(dbl_mult))`; otherwise the
  pre-loop value `kb = getModdedBaseDmg(atk) × FAC` (L10393-10394), × Sharpshot for AoE (L10405).

### 3.6 Direct-hit multiplier `Jb` (L10860-10880)
```
Jb = multishotTier × (1 + cust(dmgVulnerability)) × ta × Ma × crit_i × I × Q × Ia × Ha × ob × ya × na × gb × ab × pb × va × J × eb × Ua × Cb
```
| Sym | Value | Lines |
|---|---|---|
| `ob` | `1 + Σmods(slam_mult)` for melee attacks of type `s`/`hs` (Seismic Wave 2, Nira's 1.5) | L10503-10508 |
| `ya` | `strengthMult` (= ability strength/100) if `currWeapon.strengthType` is `simple` or `excalibur` (exalted weapons) | L10350-10355, L14488 |
| `na` | `currWeapon.additionalBuff` (weapon JSON) or 1 | L10356 |
| `pb` | Tenet Plinx only: `1 + aUniq(magazineSizeMult) × (moddedMag − baseMag)` | L10514-10519 |
| `va` | `1 + Σmods(multDAIKYUBroadhead)` (Spring-Loaded Broadhead; also in per-type maps, SP Q13) | L10406 |
| `J` | sniper combo: attack flag `sniperCombo` → setting value or **1.5** | L10326-10330 |
| `eb` | `1 + Σmods(electricityShardAbilityDmg)` (Archon shard "+x % Ability Damage on enemies affected by Electricity") | L10389 |
| `Ua` | Exalted Weapon only: `max(abilityCombo setting, cust(abilityCombo))` when > 1 | L10377-10388 |
| `Cb` | overguard layer only: `Σmods(mult_overguard)` if > 0 (Secondary Fortifier `WITH_COND 8` → ×8) | L10407-10408, L4764 |

### 3.7 Status damage multiplier `zb` (L10498-10502)
`zb = (1 + Σmods(status_damage) + cust(status_damage) + (heavy ? Σmods(h_status_damage) : 0)) × (1 + Σmods(multiple)) × (1 + Σmods(dmgMultAgainstFrozen)) × (1 + Σmods(vuln_status_damage) + cust)`.
Per-type additions (added to `zb`, not multiplied): Toxin `wc` (Archon shard 0.3/0.45, L5373-5382), Heat `Wc` (+ strength-scaled), Electricity `Gd`, Slash `kd = uniq`, Gas `Db` (Leaded Gas) (L10449-10464).

## 4. Status (DoT) damage per proc (L10883-11362)

Skipped entirely if the enemy is immune to "all" (`Nb`). For each status `s` in `ca`:
`lb = 1 − elemRes[s]` (enemy-specific, L10890-10893), `Ra = 1 − iDRs` (L10894), `Vsq = (1 − iDR[L])²` (L10885).

Common factor (the DoT version of `Jb`, without crit, plus `Ob`):
`P = Ob × multishotTier × (1+dmgVuln) × ta × Ma × Q × I × Ia × Ha × ob × ya × na × gb × ab × pb × va × J × eb × Ua × Cb`,
with `Ob = 3` on the **shield layer**, else 1 (L10576).

Tick base from `getTickStatusDmg(T, FAC, base, Fa)` (L9154-9204):
```
tick = c × base × (1 + elemMods(T) [Heat: cust(heatInherit) replaces it if >0] + add_toxin/addElectricity/addCold/addHeat (+ add_toxinS×strength)) × FAC × Eclipse × (1 + dbl_mult)
       c = 0.3 for Blast, 0.5 otherwise;  Eclipse = mirageBuff ? 1 + val/100 : 1
health  = calcPercentAdd(tick, typeMod(T));  shield = same (only if enemy has shields)
overguard = Void ? tick × 1.5 : tick
armor   = (calcPercentAdd(health, typeMod(T)) × (1 − DR(A × max(Fa,0)))) × (1 − iDR.armor)
```
`elemMods(T)` = `getStatValFromModElements` (L8749-8762) = sum of mod `element[T]` (base elements only appear as keys).

| Status | per-tick value `d` (crit path; `nc` uses `fa` instead of `kb`) | Summed into `Bb` | Lines |
|---|---|---|---|
| Slash | `0.35 × kb × FAC × Vsq × P × (zb + kd) × Id × crit_i × uDR × lb × Ra` — no armor, no type modifier; `Id = 1 + Σmods(finisherDmg)` if attack flag `enableFinisherModsWithSlash` | `curr_duration × d` (6 s base) | L10904-10987 |
| Toxin (not on shield layer) | `tick[L] × (1 − iDR[L]) × P × (zb + wc) × crit_i × uDR × lb × Ra` (base `kb`) | `curr_duration × d` (6 s base) | L10988-11058 |
| Heat | `tick[L] × Vsq × P × (zb + Wc) × crit_i × uDR × lb × Ra`; also stored as `proc_d/proc_nc` | `curr_duration × d` (6 s base) | L11059-11144 |
| Electricity | `tick[L] × Vsq × P × (zb + Gd) × crit_i × uDR × lb × Ra` | `curr_duration × d` (6 s base) | L11145-11218 |
| Gas | `tick[L] × Vsq × P/Ma × (zb + Db) × crit_i × uDR × lb × Ra` (`Ma` omitted); `nc` base is `fa × FAC` | **once per hit**: `gasTicksOnEnemy × d` (or 6 if none) | L11219-11290 |
| Blast | `0.3`-base tick `× Vsq × P × zb × crit_i × uDR × lb × Ra`; `nc` base `fa × FAC` | `curr_duration × d` (1.5 s truncated to 1 without duration mods) | L11291-11361 |
| others (Impact, Puncture, Cold, Corrosive, Magnetic, Radiation, Viral, Void, Tau...) | no damage; only affect timeline modifiers | — | — |

`uDR` = unique-enemy factor (§6.3); `curr_duration` = base duration × (1 + status_duration mods), `parseInt`-truncated (L9085-9092).
**Cascadia Empowered** (`dmgOnStatusEff` = 750, L5174) for each base/combined-element status:
`cd += 750 × typeMod(T) × FAC × I × Q × lb` (× armor DR on armor layer, × uniqueDR) (L10896-10903, `calcDmgOnStatusEff` L10240-10253).

## 5. Direct damage on each layer (L11364-11394)

- health / healthFull / shield / overguard: `g` as passed (per-type maps already include faction mods, Roar, Eclipse,
  Madurai (physical, not POWER_WEAPON), `multiple`, frozen, Broadhead, `dbl_mult`, 1/16 quantization and the faction
  type modifier; overguard has no type modifier except Void ×1.5; health excludes Toxin-less types when the attack has
  Toxin and the enemy has shields — SP §7).
- armor / armorFull: `g = Σ_type health[atk][type] × (type == "finisher" ? 1 : (1 − DR(A × max(Fa,0))) × (1 − iDR.armor))`,
  each nonzero entry raised to at least 1 (`armorDmgWithDecreaseArmor` L9801-9818, called L11364-11375). With non-crit perks
  the same is done on the non-crit map to give `K`.
- `Ac` (Volatile Rebound / Volatile Quick Return `glaive_explode`, L2314-2321): `Ac = (Jb / Ha) × g × 0.5` (L11376).
- `K` = Σ of the non-crit map (`reCalcWithNonCritBase(true)`, `BaseModsDmg(true) = (1+base+ADD)(1+MULT)`, L8462-8478, L10342-10346).

## 6. Path selection and assembly (switch L11400-11961)

```
Nd = some mod has base_uncrit_unstatus           (Laetum "Overwhelming Attrition": ADD 12)          L10217-10221, L11397
z  = mods with base_noncrit                      (Laetum/Phenmor "Devouring Attrition": MULT 20, chance 0.5)  L10222-10226
case Nd                                        → "non-crit" path with K for EVERY hit                L11401-11579
case z && crit_i == 1 && roll(100·z[0].chance) → "non-crit" path with K                              L11580-11760
default                                        → normal path with g                                  L11761-11960
```
Normal path (no unique enemy):
`direct = (g × Jb + Ac) × (1 − iDR[L]) × Ta`, `hb = direct + Bb` (L11957-11960), `Ta = 1` for ordinary enemies.
Non-crit paths: `direct = K × Jb × (1 − iDR[L]) × Ta`, `hb = (K × Jb + kc) × (1 − iDR[L]) × Ta` (L11575-11577, L11756-11758).

### 6.1 Xata's Whisper (`hakuXata`, option "optXata", L12075-12077) — `getXataDmg` L13226-13243
```
base = Σ_type atkM.damage[type]          (modded per-type incl. elements, no quantization, no type modifier)
mult = Jb × ta × Ca × FAC × (1 + dmgVuln)  [× 21 in both non-crit paths]          L11403, L11584, L11763
xata = round( base × mult × FAC × (L == overguard ? 1.5 : 1)
              × (L == armor ? 1 − DR(A×Fa × (1 − iDR.armor)) : 1 − iDR[L]) × val/100 )
xata = round(xata × hc); if (demolisherNecramech || dedicant) && xata > 176000: xata = round(176000 / multishot)
archon: round(getArchonDR(xata, 1, shotgun)); other DR bosses (Xc list): round(xata × uniqueDR((xata/FAC) × speed × MS × Ca × (1 − iDR[L])))
```
So Xata double-dips faction (`FAC` twice) and headshot (`ta`, `Ca` inside `Jb` via `Ma` and again explicitly);
wiki:Xata%27s_Whisper L24-25 says it double-dips faction and body-part weaknesses.

### 6.2 Toxic Lash (`toxicLash`, "optTLash") — `toxicLashDmg` L13215-13225
```
pct  = (melee ? 2 : 1) × val/100
G    = (1 − iDR[L]) × crit_i × Ob × msTier × (1+dmgVuln) × ta × Ma × Q × I × Ia × Ha × ob × ya × na × gb × ab × pb × va × J × eb × Ua × Cb
direct_hit = ceil(base × G × FAC × pct)                                  base = g (normal path) or K (non-crit paths)
tick_dmg   = ceil(base × FAC² × pct × 0.5 × (1 + Σ mod Toxin element) × G × (zb + wc))
total      = direct_hit + tick_dmg × statusDuration.ToxicLash.curr_duration (base 7.5 s × (1 + status_duration), `parseInt`-truncated → 7 with no duration mods, L5555-5561, L9085-9092)
```
Normal path also applies `hc`, rounding, Necramech/dedicant cap, archon DR, other-boss DR (L11855-11878). Timeline gets a
Toxin stack (index-based, L11879). Layer `L` and `Fa` are passed but unused inside `toxicLashDmg`. Faction ×2 on the
hit and ×3 on ticks (g already contains FAC) — matches wiki:Toxic_Lash L22.

### 6.3 Unique enemies (enemy `unique` field; `S = currEnemy.unique` truthy, L10347)
Flags L10539-10557. `uniqueDR(x)` = `swithTypeUniqueEnemyDR` (L13142-13192); DR curves themselves (L13070-13141) belong to the
unique-enemy research. How `getModelQueueDmg` feeds them:
| Enemy | Direct | Status | Lines |
|---|---|---|---|
| demolisher, amalgam, jugulus, saxum, amalgam-machinist, orphix, empyrean-corpus, bursa (`Xc`) | `Ta = uniqueDR((g/FAC) × speed × MS × Ca × (1 − iDR[L]))` in the normal formula | per-status metric `(kb/FAC) × 0.35 × Ma × speed × MS × crit × Vsq` (slash) or `getTickStatusDmg(T,1,kb/FAC,Fa) × speed × MS × crit × Ma × Vsq` → `uniqueDR(metric)`; jugulus uses `jugulusDRProc`; demolisher/jugulus heat accumulates in `Ea` across **all hits** | L11379-11394, L10905-10914, L10989-10994, L11060-11075 |
| eidolon (`Yc`) | `hb = eidolonDR(g × ta × (headshot ? Ma : 2) × 2 × crit × Ia × Ha × ya × na × gb × ab × (1+dmgVuln), speed, crit)` (returns damage, not a factor); no status added | `eidolonDR(0)=0` → statuses 0 | L11882-11899, L13078-13084 |
| acolytes (`Yb`) | `mb = g × msTier × dmgVuln × ta × Ma × I × Q × Ia × Ha × ob × ya × na × gb × ab × pb × va × (shield ? 2 : 1) × (1 − iDR)`; `acolytesMetric` → `hb = mb × moddedCritMult × acolytesDR(dpsMetric) + Bb` | each status: metric from its own value | L11900-11929, L10283-10294 |
| lephantis, hemocyte (`Zc`) | `Ta = lephantisDR((g/FAC) × 2 × crit × MS × Ma × (1−iDR), speed)`; `hb = (2 × g × crit × ta × Ma × Ia × Ha × ya × dmgVuln × na + Bb) × (1 − iDR) × Ta` | inside `hb` | L11930-11948 |
| archon (`wb`) | `hb = getArchonDR((g × Jb + Ac)(1−iDR) Ta, MS, shotgun) + Bb`; `getArchonDR(b,ms,sg) = (x/(x+1)) × 460000/ms`, `x = b·ms·(sg?0.7:1)/460000` | each status through `getArchonDR` | L11949-11956, L13265-13268 |
| demolisherNecramech (`xb`) | `hc = 0.5875` multiplies `curr_dmg` and the hit total | statuses also get `uDR = 0.5875` | L10557, L11962, L12009 |
| dedicant (`tb`) | only the 176 000 cap on Xata / Toxic Lash | — | L11406, L11857 |

### 6.4 Nightwatch Napalm (mod `nNapalm`, L2409; only for attack named **"Rocket Explosion"**) L10409-10441, L10767-10819
```
Xb = 150 × (typeMod(Heat) on health | on shield) × (typeMod(Heat) on armor)      L10413-10414
Nc = (same as Xb, see quirk)                                                        L10415-10416
jd = 1 + progenitorBonus% (#pElVal, 25-60) for tags Tenet / Kuva Lich / Coda, else 1
vc = Xb × jd × FAC × BM × Eclipse;   Dc = Nc × jd × FAC² × BM × (1 + heatMods) × Eclipse        (BM = BaseModsDmg(false))
per hit: jc = vc × min(6, remainingSeconds), ac = Dc × min(6, remainingSeconds)
         remainingSeconds = floor(queueLength − i × queueLength / hits)
         × I (viral) × Q (magnetic) × zb; armor layer: × (1 − DR(A×Fa×(1−iDR.armor))); 0 if heat-immune
```
Napalm also halves armor on every hit of that attack (`ub` in `Fa`).

### 6.5 Reconifex heat (`aUniq(reconifexHeat)`) L11964-11984
`V = curr_dmg × Ub`; `X = round(V/2) × FAC × (1 − elemRes.heat) × (1 − iDRs)` is added to `status.heat.d` and
`instance_d`; `curr_dmg += V`; `mc = 6 × status.heat.d`. Armor is halved for all hits of the attack.

### 6.6 Primary Debilitate damage L11431-11462 / L11612-11643 / L11791-11822, L11985-11993
For each element in `Gc`: `ceil(getTickStatusDmg(T, FAC, base)[L] × (1 − iDR[L]) × crit × Ob × … × Cb × FAC × (zb + Sb[T]))`
(`base = kb` normal path, `fa` non-crit paths); only Heat/Toxin/Electricity get a value; `pc = Σ value × 6`.

### 6.7 Hit total (L11998-12011)
```
totalDmg_i = ( hb + xata + (TL.direct_hit + TL.tick_dmg × 7.5) + pc + napalm(jc + ac) + mc + cd ) × hc
curr_dmg_i = direct part (display); toxinDmg_i = curr_dmg_i × W, W = toxinShare of layer total (L10337-10341, L11963)
```

## 7. Consolidated per-hit formula (normal path, ordinary enemy, layer L)

```
totalDmg_i = hc × [
    G_L × Jb_i × (1 − iDR_L)                                   # direct
  + 0.5 × G_L × Jb_i / Ha × (1 − iDR_L)          [glaive explode]
  + Σ_{s ∈ statuses_i} dur_s × Tick_s,i                        # DoT credited entirely at the applying hit
  + Xata_i + ToxicLash_i + Debilitate_i + Napalm_i + Reconifex_i + Cascadia_i ]

Jb_i  = msTier × (1+cust dmgVuln) × HSinc × (crit_i>1 ? 1 : HSmult) × crit_i × Viral_i × Mag_i × stance_i × heavyCombo_i
        × slam × strength(exalted) × additionalBuff × firstShot_i × lastShot_i × plinx × broadhead × sniperCombo × elecShard × abilityCombo × overguardMult
crit_i = [ setCritDmgToHit tier value (rolled with per-hit CC incl. 5%/Puncture stack + Enervate) ]
         × (1 + SpitefulDefilement if <3 stacks) + Cold bonus (0.05/stack, +0.5 at 10) → stealth transform
Viral_i = 2 + 0.25(viralStacks_i − 1) on health/armor;  Mag_i = 2 + 0.25(magStacks_i − 1) on shield/overguard
G_L   = Σ_type map_L[type] (1 − elemRes) ; armor: Σ_type health[type] (1 − DR(A·Fa_i))(1 − iDR_armor), Fa_i from Puncture-mod/Corrosive/Heat/armor mods
Tick_s,i = c_s × statusBase × FAC × (1+elemMods_s) × Eclipse × dblMult × typeMod_s(L) [armor: typeMod² × armorDR]
           × (1−iDR_L)^(1 or 2) × P_i × (zb + typeBonus_s) × crit_i × lb_s × (1 − iDR_status)
statusBase = Σ atk.base × (additive or multiplicative CO, Shiver, Sharpshot) × BM × FAC × dblMult      (Slash: 0.35×statusBase×FAC, no typeMod/armor)
```

## 8. Arcane and conditional stacks over time

- Nothing in `getModelQueueDmg` ramps arcane stacks over time. Arcanes/Galvanized/conditional mods are static values in the
  mod table, at max stacks under `WITH_COND` when "all mod conditions" (`isMaxCond`, default on) is set — e.g. Primary
  Merciless `WITH_COND {base: 3.6}` = 12 × 30 % (L4594-4600), Arcane Avenger `WITH_COND {flat_crit_chance: 0.45}` (L5153-5158).
- Dynamic per-hit state that **is** simulated: status stacks (timeline, §1), Secondary Enervate CC ramp/reset (§3.2),
  Tau status-chance ramp (`procStatusArray` L9994-10001: active Tau stacks q add `0.1·q` to the status-chance multiplier, `getFinalStatusChance` L9891-9895), melee heavy combo
  (`addTimelineMeleeCombo` L12158-12181: starts at `20 × (combo − 1)` hits [≥ initialCombo], `comboMult = int(1 + hits/20)`,
  every `int(0.8 × hits/queueLength) + heavyHits` hits the counter becomes `max(hits × comboEfficiency(≤0.9), initialCombo)`, 0 if < 20),
  Sagek passive chain, napalm remaining-time, Melee Duplicate extra hits.
- CO status count `fc` is static (number of damage types, SP Q4); `critFlatChancePerStatuses` uses the same static count.

## 9. Magazine, reload and first-shot handling (outside this function: scheduler in `createDmgQueue`, L12251-12406, L12459-12525)

- Effective magazine `= (incarnon ? incMagazineSize : magazineSize) / (1 − ammoEff)` (ammoEff capped at 0.999999; infinite
  if `hasInfiniteMagazine`); ammo per shot 0.5 for beams, `ammoCost` from JSON, `"all"` = whole magazine (L12459-12481, L12264-12278).
- Reload: per-round reloaders `reloadDelay/(1 + reloadTime mods) + mag/reloadRate`, else modded `reloadTime` (L12279-12287).
- Fire rate `Ca`: bursts `S / ((S−1) × burstDelay + 1/rate)`; bows and "Balefire Charger (Hildryn)" with charge, and any other
  charge weapon: `1 / (charge + reload)`; Lanka: `1 / charge` (L12288-12296). The first shot is delayed by the charge time (L12351).
- Shots are laid out until `queueLength`; when shots-per-mag are used up or ammo < cost, a reload of duration `Ea` is
  inserted (L12333-12349). Beam multishot is not expanded (§2).
- First-/last-shot multipliers: §3.5. No other first-shot effect exists in the per-hit loop (`isAfterReload`,
  `isChargedShot` are recorded in `shotInfos` but never read by the damage code).
- Aggregation: `getDmgPerSecond` spreads `totalDmg` over `queueLength` one-second buckets by **hit index** (uniform), not by
  hit time; queue DPS = median of the buckets (L12787-12805).

## 10. Complete list of name-specific branches in L10311-12014

| Name / key | Lines | Effect |
|---|---|---|
| Attack `sniperCombo` flag | L10326-10330 | ×1.5 (or setting) sniper combo |
| `strengthType` simple/excalibur | L10350-10355 | ×ability strength |
| `additionalBuff` weapon field | L10356 | ×value |
| Attack `no_headshot_mult` | L10358-10367 | disables headshot multipliers |
| Heavy attack (`isHeavy` / option) | L10368, L10587 | heavy combo `Ha`, `h_status_damage`, heavy CC |
| `decr_armor_by_punc` | L10369, L10806-10810 | armor −x per Puncture stack |
| Volatile Rebound / Quick Return (`glaive_explode`) | L10370, L11376 | +50 % hit as explosion |
| shotgun categories | L10371-10375 | archon DR 0.7 factor, acolyte metric ×0.5 |
| Cascadia Empowered (`dmgOnStatusEff`) | L10376, L10896-10903 | +750 per elemental status |
| Exalted Weapon type | L10377-10388 | ability combo multiplier |
| `dmgVulnerability` custom stat | L10378 | ×(1+x) everything |
| Archon shard electricity ability dmg | L10389 | ×(1+x) everything |
| Charged/Primed Chamber, Synth Charge | L10390-10391, L10775-10776 | first/last shot multipliers |
| attack `shot_type == "AoE"` | L10396-10405, L10650 | no CO, Sharpshot on status base, no Puncture CC |
| Longbow Sharpshot (`multSharpshot`) | L10404-10405 | ×4 status base (AoE) / via CO term |
| Spring-Loaded Broadhead | L10406 | ×(1+x) in `Jb` |
| Secondary Fortifier (`mult_overguard`) | L10407-10408 | ×8 on overguard |
| Nightwatch Napalm + attack "Rocket Explosion"; tags Tenet / Kuva Lich / Coda | L10409-10441 | napalm damage, armor halving |
| `crit_after_status` (Gotva) | L10442, L10674-10678 | forced CC after status |
| `dbl_mult` (Mecha Empowered) | L10444, L10843 | status base ×(1+x) |
| Archon Continuity / Vitality | L10445-10446 | passed to status re-rolls |
| Primary Debilitate | L10447, L10621-10639, §6.6 | extra base-element procs |
| Melee Duplicate | L10448, L10697-10741 | duplicate hit on tier-1 crits |
| Toxin/Heat/Electricity/Slash/Gas status-damage keys | L10449-10464 | per-type status bonus |
| Melee Careen (`dmgMultAgainstFrozen`) | L10466, L10501 | ×2.5 status damage, unconditional |
| Burning Hate? (`vuln_status_damage`, L208) | L10467-10469 | ×(1+x) status damage |
| Spiteful Defilement (`flat_crit_with_status`) | L10470, L10752-10760 | ×2 crit multiplier |
| `enableFinisherModsWithSlash` attack flag | L10471-10473 | finisher mods on Slash DoT |
| `disableBodyCrit` | L10474, L10693 | no crits unless headshot |
| Hunter Munitions (by mod name) | L10475, L10761-10766 | slash on crit |
| Melee slams (`s`/`hs`, name contains "slam") | L10503-10513 | slam mods; no CO; no Duplicate |
| **Tenet Plinx** (weapon name) | L10515-10519 | magazine-size multiplier |
| `critFlatChancePerStatuses` | L10522 | flat CC × status types |
| Enemy uniques demolisher, amalgam, jugulus, saxum, amalgam-machinist, orphix, empyrean-corpus, bursa, acolytes, eidolon, lephantis, hemocyte, archon, demolisherNecramech, dedicant | L10539-10557, §6.3 | DR handling |
| `addPunctureStatus` | L10558, L10640-10647 | +Puncture stacks on this hit |
| Secondary Cryogenic | L10559, L10742-10751 | +3 Cold on Puncture |
| Incarnon flat (`flat_change_dmg`) | L10560-10563, L10839-10842 | flat split in status base |
| attack `isCoMult` | L10564, L10830-10850 | multiplicative CO |
| Reconifex (`reconifexHeat`) | L10565, L11964-11984 | heat damage + armor halving |
| Toxic Lash option | L10571, §6.2 | extra hit + ticks |
| Secondary Enervate | L10573, L10673, L10695-10696 | CC ramp |
| Sagek passive, Secondary Encumber | L10579-10620, L10694 | forced status re-rolls |
| `incMaxStacks_Corrosion` | L10582 | Corrosive cap +2 in timeline extension |
| Vigilante (`double_crit`) | L10680-10683 | +1 tier chance |
| External Viral, Stealth, Xata, Eclipse (via getTickStatusDmg / maps), Roar (via FAC), Nourish (Viral element in `newElements` L13019-13026), Madurai (maps only) | various | see §3, §6 |
| Laetum/Phenmor non-crit perks (`base_uncrit_unstatus`, `base_noncrit`) | L10322-10324, L11400-11760 | non-crit paths |

## Suspected inspiration quirks
- **QM1** A hit benefits from its own status procs: `timelineStatus` counts hit k's statuses at index k, so its own Viral/Magnetic/Corrosive/Heat/Cold/Puncture modify its damage (L10093-10136, L10780-10825). Game applies procs after the hit (verify vs wiki research).
- **QM2** Viral `I` and Magnetic `Q` are declared outside the loop and only set when the status is present; after it expires they keep the last value (L10319-10321, L10780-10804).
- **QM3** DoT ticks except Toxin use `(1 − iDR)²` (`V`), Toxin `(1 − iDR)` once (L10885, L11000).
- **QM4** All DoTs ×3 on the shield layer (`Ob`, L10576); purpose unknown.
- **QM5** Gas DoT omits the non-crit headshot multiplier `Ma`, credits only one gas proc per hit and multiplies by current gas stacks instead of duration (L11230-11289).
- **QM6** Gas and Blast `nc` bases use `fa × FAC` (faction applied an extra time) (L11227, L11299).
- **QM7** DoT on armor applies the faction type modifier twice (`calcPercentAdd(health, typeMod)` where health already has it) (L9198-9202).
- **QM8** Cold crit bonus is +0.05/stack (wiki +0.10 first stack, +0.5 at 9; Frozen +1.0) and is added to the final tiered multiplier, not to CDM before tier scaling (L10821-10825; wiki:Damage__Cold_Damage L20-23).
- **QM9** Puncture stacks last 6 s in `statusDuration` (wiki 10 s) (L5547; wiki:Damage__Puncture_Damage L18).
- **QM10** Hunter Munitions chance is `0.3 × CC` on crits instead of 30 % per crit (L10763-10764; wiki:Hunter_Munitions L2-21).
- **QM11** Overwhelming Attrition (`base_uncrit_unstatus`) applies its ×13 base to **every** hit (crits and status hits too), and DoT uses the non-crit base `fa` (L11401, L11575).
- **QM12** Xata in non-crit paths uses a hard-coded ×21 (Devouring Attrition's 1+20) also for Overwhelming Attrition (L11403, L11584).
- **QM13** Toxic Lash other-boss branch reads undefined `Za.direct_dmg` → NaN, and writes to `xataTotal` (L11866-11877).
- **QM14** Non-crit paths multiply status damage `kc` by `(1 − iDR) × Ta` again; normal path does not (L11575 vs L11958).
- **QM15** Non-crit paths drop glaive explosion `Ac`; the Overwhelming path ignores the actual non-crit/non-status condition.
- **QM16** Armor pass always derives `g`/`K` from `currWeapon.health` (Toxin-only map when shields exist), so `armorFull` equals `armor` for Toxin weapons vs shielded armored enemies (L11365-11373).
- **QM17** Armor-layer direct damage skips enemy-specific `elemRes` (summed raw, L11369-11373) while other layers apply it (L9110-9119).
- **QM18** Napalm: `Nc` is overwritten with `Xb`-based value (75 never used; armor factor squared) (L10415-10416); progenitor bonus `jd` scales napalm on Tenet/Kuva/Coda launchers (L10417-10440); DoT ×FAC².
- **QM19** Reconifex: `mc = 6 × status.heat.d` re-counts an ordinary heat proc already in `Bb`; the direct part `V` is only in `curr_dmg`, not in `totalDmg`; `hc` applied twice (L11962-11983, L11998-12009).
- **QM20** Necramech Demolisher: status damage × 0.5875 twice (uDR and `hc`) (L13139-13141, L12009).
- **QM21** Spiteful Defilement doubles the whole crit multiplier instead of +100 % critical damage, and counts total stacks (`Σ ticks`) rather than distinct status types (L10752-10760; perk text in samples/weapon-Phenmor.json).
- **QM22** Melee Careen and Archon electricity-shard bonus are unconditional (no frozen / electricity check) (L10466, L10389; wiki:Melee_Careen L46).
- **QM23** Melee Duplicate clone sets `crit = Da.tier` (overwritten when processed) and its statuses do not propagate to later timeline entries; `addPunctureStatus` likewise only touches `d[i]` (L10717-10740, L10640-10647).
- **QM24** DoT damage is credited in full (`duration × tick`) at the applying hit; it is not truncated at the end of the window, and per-second buckets are spread by hit index, not time (L10982-10986, L12787-12801).
- **QM25** Fractional crit, multishot and status chances are rounded to whole percents before rolling (L10679, L10054, L10012).
- **QM26** Charged weapons' fire rate is `1/(charge + reload)` for every charge weapon (except Lanka), and the magazine reload is still inserted on top (L12290-12296, L12333-12349).
- **QM28** `curr_duration = parseInt(duration × (1 + status_duration))` truncates Blast 1.5 → 1 and Toxic Lash 7.5 → 7 (and any modded duration to whole seconds), and these durations multiply the credited DoT/Blast damage (L9085-9092, L11356-11360, L12002).
- **QM27** Eidolon direct uses `(headshot ? Ma : 2) × 2` and the Overwhelming-path Eidolon formula omits `ta` (L11499-11510 vs L11680-11692).

## Open questions
- Purpose of `Ob = 3` on shields and of `(1 − iDR)²` on DoTs (QM3/QM4): deliberate model of shield gating / boss DR, or bugs?
- Which weapons carry `sagekPassive`, `reconifexHeat`, `crit_after_status`, `disableBodyCrit`, `addPunctureStatus`, `decr_armor_by_punc`, `magazineSizeMult`, `critFlatChancePerStatuses`, `status_damage_heat/electricity/slash`, `enableFinisherModsWithSlash`, `additionalBuff`, `strengthType` — not in the static mod table or samples; needs the weapon dump.
- Which mods carry the plain `armor` key used as `qa` (L10348) — not traced.
- Whether game Gas DoT scales with `elemMods("Gas")` (inspiration: only mods with a Gas element key count; wiki:Damage__Gas_Damage L44 says Gas mods only).
- Meaning of the eidolon `×2 ×2` and lephantis `×2` constants (likely weak-point multipliers) — not documented in the bundle.
- `getChangedComboMult` reset cadence `int(0.8 × hitsPerSecond) + heavyHits` — intended model of "heavy attack every ~second"? Not explained in code.
- Whether the user-entered `queueLength` also changes `statusDuration` crediting (it does not; DoT always credited in full).
