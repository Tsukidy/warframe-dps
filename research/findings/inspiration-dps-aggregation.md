Status: draft

# Inspiration: hit queue construction and DPS aggregation

Source: `research/external/inspiration/min.pretty.js` (cited as **L** + line). Function names are the bundle's own; local
variables are minified, so every formula is rewritten with readable names next to its line range. Wiki citations point at the
cached wikitext in `research/external/cache/wiki/` (copies of wiki.warframe.com pages). Shot schedules in the worked examples
were produced by a hand transcription of the scheduler in Python (`wfsched.py` in this agent's scratchpad, not committed);
they were not produced by running the inspiration's code.

Related findings (not repeated here): per-hit damage and stat maps in `inspiration-stat-pipeline.md`; status and crit rolling
and per-hit status damage in `inspiration-status-and-procs.md`; UI controls in `inspiration-feature-inventory.md`.

---

## 1. Pipeline overview

```
reCalc()                                   L9638-9778
  addDmgToBase / swapDmgType / addWeaknessDmgType / newElements()   L9640-9661  (element merge, see §13)
  reCalcWithTypeBase(false) -> currWeapon.health/healthFull/armor/armorFull/shield/overguard  L9662-9671
  reCalcDisplay()                          L9140-9153  (creates one result row per attack and layer, then:)
    createDmgQueue()                       L12250-12786
      for each attack x (independently):
        build hit list ("fullModel"):  guns -> shot schedule b() + procMultishotArray      L12458-12525
                                       melee -> getMeleeCountHits10s + even spacing        L12436-12457
        melee only: procMeleeComboAttackEffect (stance mult + stance statuses)             L12527-12531
                    addTimelineMeleeCombo (heavy only; per-hit combo multiplier)
        makeRNDMaskStatusArray, procStatusArray (status rolls)                            L12532-12533
        setForceProcs, setProcsITS (forced / impact-triggered procs)                       L12536-12537
        timelineStatus (active stacks per hit)                                             L12546-12547
      optional "combine attack effects": merge timelines of grouped attacks               L12540-12553
      for each attack k and each defence layer present:
        getModelQueueDmg(...) -> arrInfo[layer][attackName][i] = per-hit record           L12559-12693
        queue_<layer> = median(getDmgPerSecond(arrInfo[layer][attack]))                    L12592-12693
        TTK = calcTTKWithStatusUnified(...)                                                L12694-12715
        createDamageInfoHTML(...)  -> the numbers shown                                    L12716-12784
  build share URL (skipped while isQueueRun)                                               L9673-9777
```

Every call of `reCalc` draws a **new random queue** (UI text: "Any change starts a new random queue", index.txt L47;
algorithm description index.txt L128-145). The shot *times* are deterministic; the randomness is in multishot pellet
counts, crit tiers and status procs (§11).

Globals: `queueLength = 20` (simulated seconds, L5496), `isQueueRun = false` (L5464), result accumulators `dpsQueue`,
`TTK_Queue`, `MedianTotal_Queue`, `TotalDmg_Queue` (L5475-5478), `maxCountProc = 0` (L5485; 0 means "no cap").

---

## 2. Gun shot schedule: inner function `b(attack, T, opts)` of createDmgQueue (L12251-12406)

### 2.1 Inputs (L12251-12278, caller L12459-12488)

| name | meaning | source |
|---|---|---|
| `T` | window length = `queueLength` | L12482 |
| `rate` (`W`) | modded attack `speed` (fire rate), `|| 1` | L12260; modded at L8919-8939: `base*(1+speed+custom+zoom+attackUnique)*(1+speed_mult+attackUnique speed_mult)` |
| `burst` (`U`) | `burst_count > 1` | L12259 |
| `S` / `burstDelay` (`qa`) | burst count / modded burst delay | L12261-12262; delay modded at L8941-8952 |
| `charge` (`ba`) | modded `charge_time` if the attack has one, else 0 | L12263; modded at L8981-8991: `base/(1+charge_time mods)` |
| `isInc` (`aa`) | attack has `isInc` (incarnon form) | L12459 |
| `infinite` (`ja`) | weapon `hasInfiniteMagazine` | L12463 |
| `effMult` (`ra`) | ammo-efficiency multiplier `1/(1-ammoEff)` | L12464-12471 |

Ammo efficiency (L12464-12471):
```
ammoEff = isInc ? 0 : mods.ammoEff + unique.ammoEff + attackUnique.ammoEff
costMult (M) = max(0, 1 - ammoEff)               // only used for the optimizer's ammoSchedule
if ammoEff >= 1: ammoEff = 0.999999
effMult (z)  = 1 / (1 - ammoEff)
```
So ammo efficiency is applied deterministically as a magazine-size multiplier (expected value), not as a per-shot roll.

Magazine and ammo cost (L12264-12278):
```
mag      = (infinite && !isInc) ? INF
         : (isInc && incMagazineSize > 0) ? incMagazineSize
         : currWeapon.magazineSize                         // modded, L9007-9018: round((base+baseMagazineSize)*(1+Σ)+add_magazineSize)
ammoCost = (weapon compTags has "BEAM" || attack.isBeam) ? 0.5
         : attack.ammoCost ?? 1
if attack.ammoCost == "all": ammoCost = currWeapon.magazineSize; mag = currWeapon.magazineSize
infiniteEff = infinite && !isInc && ammoCost != "all"
mag = infiniteEff ? INF : mag * effMult                    // float, not floored
```

Reload time `Ea` (L12279-12287):
```
if weapon has reloadRate and reloadDelay (battery weapons):
    reload = initial.reloadDelay / (1 + mods.reloadTime + unique.reloadTime) + mag / currWeapon.reloadRate
else:
    reload = currWeapon.reloadTime        // modded: base / (1 + mods + custom + unique), L9029-9041
```
Note `mag` here already includes the ammo-efficiency multiplier (see quirk Q6).

Effective shot rate `Ca` (L12288-12296):
```
Ca = rate
if burst: Ca = S / ((S-1)*burstDelay + 1/rate)
if (category "primary-bow" || name == "Balefire Charger (Hildryn)") && hasCharge: Ca = 1/(charge + reload)
else if name == "Lanka":                                                          Ca = 1/charge
else if hasCharge:                                                                Ca = 1/(charge + reload)
```
Shots per magazine (L12298): `perMag = (infiniteEff && reload == 0) ? INF : max(1, floor(mag / ammoCost))`.

### 2.2 Planned shot count `Pa` (L12299-12321)

`Pa` is only a loop bound; the loop in §2.3 is authoritative.
```
if ammoCost == "all" and mag finite:
    if T < charge: return no shots
    Pa = 1 + floor((T - charge) / reload)          // inner ternary at L12310 always takes `reload` because K is true
else:
    magTime = perMag / Ca
    if perMag == INF or T < magTime:
        Pa = floor(T * Ca)                         // (the "0 -> 1" fix-up at L12314 is dead code: floor(T*Ca)=0 implies T < 1/Ca)
    else:
        rest = T - magTime; cycle = reload + magTime; n = floor(rest / cycle)
        Pa = perMag*(1+n) + floor((rest - n*cycle) * Ca)
```

### 2.3 Event loop (L12322-12384)

State: `cursor` (`rb`, earliest time of next action) = 0, `ammo` (`Hb`) = mag, `shotsInMag` (`Ga`) = 0, `burstIdx` (`eb`) = 0.
```
for i in 0 .. Pa-1:
    if mag finite and not infiniteEff and (shotsInMag >= perMag or ammo < ammoCost):
        if cursor + reload > T: stop
        record reload {time: cursor, duration: reload}; cursor += reload; ammo = mag; shotsInMag = burstIdx = 0
    t = cursor
    if hasCharge and i == 0: t += charge                      // ONLY the first shot pays the charge explicitly
    if burst and burstIdx > 0: t += burstDelay
    if t > T: stop                                            // a shot exactly at T is kept
    record shot {time: t, magazineNumber, magazineIndex, isBurst, burstIndex, isChargedShot: hasCharge && i==0, ...}
    shotsInMag++; ammo -= ammoCost (if finite)
    if burst: burstIdx++; if burstIdx >= S: cursor = t + 1/rate, burstIdx = 0 else cursor = t
    else:     cursor = (ammoCost == "all") ? t : t + 1/Ca
    if cursor > T and i+1 < Pa: stop
after loop: if magazine empty and cursor <= T: record a "future reload" marker (duration 0, isFutureReload) L12386-12398
return {shotTimes, shotInfos, reloadInfos, totalReloads, totalShots}
```
Consequences:
- First shot at t = 0 (or t = charge for charge weapons). A reload begins one shot interval (1/Ca) after the last shot of a
  magazine, not immediately at the last shot (L12381 then L12333-12344).
- Reload in progress at the window end: the reload is not started and no further shots occur (L12334).
- Semi-auto weapons: no special case; the arsenal fire rate is used as an achievable rate (no trigger cap). No `trigger`
  field is read anywhere in the calculation (only in mod filtering, L7204).
- Beam / continuous weapons: one "shot" per tick at the modded fire rate, costing 0.5 ammo (wiki Continuous_Weapon.wiki L17-18
  agrees: "Consumes 0.5 ammo per tick ... Each tick is equal to 1/MODDED_FIRE_RATE"). The beam damage ramp-up (10% to 100%
  over 0.8 s, Continuous_Weapon.wiki L145) is **not modelled** (no "ramp" code; `isBeam` only read at L12257, L12430).
- `ammoCost == "all"`: one shot per reload cycle, fire rate ignored (shots at 0, reload, 2*reload, ...).

### 2.4 Worked schedules (T = 20 s, transcription of L12251-12406)

| case | stats | shots | shot times / reload starts |
|---|---|---|---|
| Furis base | 10/s, mag 35, reload 1.4 | 144 | reloads at 3.5, 8.4, 13.3, 18.2; last shot 19.9 |
| Furis +60 % fire rate | 16/s, mag 35, reload 1.4 | 209 | reloads at 2.19, 5.78, 9.36, 12.95, 16.54; last shot 20.0 (shot exactly at T kept) |
| Furis Incarnon Form (beam) | 12/s, incMag 280, cost 0.5 | 240 | 560 ticks per magazine, no reload in 20 s |
| auto, mag fills window | 10/s, mag 200 | 200 | 0.0 ... 19.9 |
| mid-reload end | 10/s, mag 15, reload 2 | 90 | reloads 1.5, 5.0, 8.5, 12.0, 15.5; reload at 19.0 skipped |
| ammo efficiency 50 % | 10/s, mag 40 (x2 = 80), reload 1.4 | 173 | reloads at 8.0, 17.4 |
| burst | 2 bursts/s, 3 x 0.05 s, mag 24, reload 2 | 72 | 0, 0.05, 0.10, 0.60, 0.65, 0.70, ...; reloads 4.8, 11.6 |
| bow | charge 1.2, reload 0.6, mag 1 | 8 | 1.2, 3.6, 6.0, ... (interval 2.4 = charge + 2 x reload, quirk Q1) |
| charge, non-bow | charge 2, reload 2, mag 5 | 5 | 2, 6, 10, 14, 18 (interval = charge + reload, quirk Q2) |
| Lanka | charge 1, reload 2, mag 10 | 18 | 1, 2, ..., 10, reload at 11 |
| slow semi | 0.3/s, mag 5, reload 1 | 5 | 0, 3.33, ..., 13.33; the 6th shot at 17.67 is cut by `Pa` (quirk Q3) |
| slow semi, T = 1 | 0.3/s | 0 | `Pa = floor(0.3) = 0` (quirk Q3) |

Wiki comparison (Fire_Rate.wiki L19-56, L172-185): burst formula `S / (1/rate + (S-1)*delay)` matches L12289. Bows:
effective rate `1/(charge + reload)` matches L12293, but the loop then adds another reload per shot. Non-bow charge weapons:
wiki `1/(charge + 1/fireRate)`; inspiration uses `1/(charge + reload)`. Lanka `1/charge` matches.

### 2.5 Ammo schedule for the optimizer (L12491-12500)

When `isQueueRun` and the attack is not incarnon, not infinite, not battery (`reloadRate`) and the weapon is not tagged
BATTERY or POWER_WEAPON: `currWeapon.info[attack].ammoSchedule = {times: shotTimes, cost: ammoCost * max(0, 1-ammoEff)}`.
Read by the optimizer (L15766-15795) as `ammoWindow = cost * shots` and `ammoToKill = cost * (#shots with time <= TTK)`.

---

## 3. Multishot to projectiles

### 3.1 Non-beam: `procMultishotArray(shots, MS, isBeam)` (L10050-10062)
```
MS     = attack.multishot ?? weapon.multishot ?? 1        // modded, L8992-9015; createDmgQueue L12431-12435
whole  = int(MS); fracPct = round(frac(MS) * 100)         // fraction quantised to whole percent
for each shot: pellets = whole + (isProc(fracPct, 100) ? 1 : 0)       // isProc(p,100) = Math.random()*100 < p, L9819-9821
```
Each pellet becomes a hit object `{crit: 1, statuses: [], time: shotTime, shotIndex}` (L12509-12518). All pellets of a
shot share its timestamp. Every pellet gets independent crit and status rolls later.

### 3.2 Beam: one hit per tick, multishot as a per-tick damage tier
`procMultishotArray` returns a mask of 1s for beams (L10053). `procStatusArray` (L10002-10010) then rolls per hit
`multishotTier = int(MS) + (isProc(round(frac(MS)*100),100) ? 1 : 0)` and multiplies that hit's status chance by it; the
damage factor `Jb` starts with `multishotTier` (L10860-10880). Matches wiki: beam multishot is merged into one instance with
summed damage and status chance (Continuous_Weapon.wiki L21-22). Non-beam hits have `multishotTier = 1`.

### 3.3 First/last-shot marks: `makeBulletsPositionMark(perMag, pelletMask)` (L12182-12196), caller L12519-12524
Only computed if `f_multiple > 0` (Charged Chamber 0.4, Primed Chamber 1.0; L2160-2169) or `l_multiple > 0` with
`perMag > 6` (Synth Charge 2; L2172-2175). `perMag = max(1, floor(mag*effMult/ammoCost))` (L12472-12481).
```
walk shots in index order in groups of perMag shots (last group may be partial):
    every pellet of the group's first shot -> first[pelletIndex]
    every pellet of the group's last shot  -> last[pelletIndex]
```
In getModelQueueDmg: `firstMult = first has i ? 1 + f_multiple : 1`, `lastMult = last has i ? 1 + l_multiple : 1`
(L10775-10776), both factors of `Jb` (L10873-10874). Grouping is by shot index, not by the scheduler's reload records.

---

## 4. Melee timeline

### 4.1 Hit count: `getMeleeCountHits10s(stanceComboName, attack)` (L10137-10216)
Despite the name it counts hits in `queueLength` seconds. `stance = modsPanel[9]?.actions?.stances?.[name]` (slotted stance
mod, combo chosen in `#stanceAttackList`); `def = default_MeleeStances[weapon.name] ?? default_MeleeStances[compTags[0]]`
(table at L6302 onward, keyed e.g. `NIKANAS_STANCE: {total: 1.77, hits: ["1","2"], statuses: ["",""], heavy: {total: 2.9,
hits: ["5","5"], statuses: [slash, slash]}}`). `heavy = isNeedCalcHeavy(attack, is_Heavy)` (L13253-13258: the attack's
own `isHeavy` wins, else the global Heavy option). `slam = attack.type in {"s","hs"}`.
```
speed = attack.speed (modded attack speed); windUp = currWeapon.windUp (modded: base/(1+windUp mods), L9059-9063)
if compTags[0] in default_MeleeStances:
    if attack.type == "h" or (no type and heavy):  cycle = def.heavy.total / speed + windUp ; hitsPerCycle = len(def.heavy.hits)
    elif no type:                                  cycle = def.total / speed               ; hitsPerCycle = len(def.hits)
    if slam:                                       cycle = 1 ; hitsPerCycle = 1             // NOT divided by speed
    hits = round(T * hitsPerCycle / cycle)
else:
    hits = speed * T                                // no rounding; speed read as hits per second
if a stance mod is slotted (overrides the above):
    slam  -> total = 1, hitsList = [1]
    heavy -> total = def.heavy.total, hitsList = def.heavy.hits     // heavy always uses the default table, not the stance
    else  -> total = stance.total, hitsList = stance.hits
    cycle = heavy ? total/speed + windUp : total/speed
    hits = round(T * len(hitsList) / cycle)
```
`total_shots_without_miltishot` is used; `total_shots` (x multishot) is unused by createDmgQueue.

### 4.2 Hit times (createDmgQueue L12436-12457)
Hits are spaced evenly over the window: `t_i = i * T / hits` for `i = 0 .. hits-1` (non-integer `hits` from the
no-stance branch yields `ceil(hits)` hits). Then `procMultishotArray(hits, MS, isBeam)` adds extra hits at the same time if
the melee attack has multishot. There is no per-hit stance timing; only the total cycle time matters.

Example (Nikana Prime sample, speed 1.08, windUp 0.5, no stance mod): light cycle `1.77/1.08 = 1.639 s`, hits
`round(20*2/1.639) = 24`, one every 0.833 s; heavy cycle `2.9/1.08 + 0.5 = 3.185 s`, hits `round(20*2/3.185) = 13`.

### 4.3 Stance multiplier and stance statuses: `procMeleeComboAttackEffect(hits, comboName, attackInitial)` (L12115-12150)
Source list: heavy -> default heavy (`hits`, `statuses`); else slotted stance combo; else default light combo. For slam
types `hitsList = [1]` (L12142) but `statuses` still comes from the chosen list. Assignment cycles through the list per hit
entry (including multishot copies):
```
p = 0
for each hit: hit.melee_mult = hitsList[p]; if statuses[p] != "": hit.statuses += statuses[p]; hit.stanceStatuses += ...
              p = (p < len(hitsList)-1) ? p+1 : 0
```
`melee_mult` becomes the factor `Ia = Number(melee_mult)` in `Jb` (L10586, L10868). Skipped for attacks with `noStanceFP`
(L12528). For slams `p` never advances, so `statuses[0]` is forced on every slam hit (quirk Q11).

### 4.4 Heavy-attack combo timeline: `addTimelineMeleeCombo(hits)` (L12158-12181), only when the attack is heavy (L12530)
```
initial   = max(mods.initialCombo + custom.initialCombo + unique.initialCombo, 0)
initial   = min(initial, 20 * combo_counter)        // combo_counter = weapon.maxHeavyCombo or 11 (L5479-5480, L8850-8852)
combo     = max(20 * (startFrom - 1), initial)      // startFrom = "Heavy attack ... start from" (#optHeavyStartCounter, default 12, 1..13), L12043-12045
perSec    = int(0.8 * len(hits) / T)                // hits per 0.8 s (truncated)
heavyHits = len(default heavy hits for weapon or compTag) else 1
countdown = perSec + heavyHits
for each hit:
    if countdown == 0: countdown = perSec + heavyHits; combo = getChangedComboMult(combo, initial)
    if combo < 20: combo = 0
    hit.comboMult = int(1 + combo / 20)
    countdown--
getChangedComboMult(c, initial) = (x = max(c * eff, initial)) >= 20 ? x : 0          // L12154-12157
eff = clamp(mods.melee_combo_eff, 0, 0.9) || 0                                         // getCurrEfficiency L12151-12153
```
So combo is spent once per `perSec + heavyHits` hits; after each spend it keeps `eff` of itself (Heavy Attack Efficiency,
capped at 90 %) and is floored at Initial Combo; it never grows. `comboMult` is used:
- as damage factor `Ha` in `Jb`, only if melee and heavy (L10587, L10869);
- in crit chance: `(comboMult - 1) * crit_chance_per_combo` (L8219-8225) for heavy / heavy-slam hits;
- in status chance: `(comboMult - 1) * status_chance_by_combo` (L9906-9910) and passed by procStatusArray (L9997-10001).

Wiki (Melee_Combo.wiki L59, L88, L93): multiplier 2x at 20 hits, +1x per 20 hits, 12x at 220 (Venka Prime 13x at 240);
Heavy Attack Efficiency keeps a percentage of current combo (L192); Initial Combo regenerates at 40 combo/s (L135, L194).
`int(1 + combo/20)` matches the tier rule. Combo gain from hits (stance multiplier points, Melee_Combo.wiki L26), the
5 s combo duration and the 40/s Initial Combo regeneration are not modelled (regeneration is treated as instant).

### 4.5 Light attacks: "Light attacks combo multiplier" option (`optMeleeCombo`, L12047-12050)
No per-hit combo for light attacks. The option only sets the constant combo used by per-combo crit and status bonuses
(Blood Rush, Weeping Wounds style stats):
```
comboForBonus = (hit is heavy/heavy-slam with comboMult>0) ? comboMult - 1
              : optMeleeCombo set and val > 0 ? val - 1
              : combo_counter                   // default 11 => 12x assumed when the option is off
```
(L8219-8224 crit, L9906-9910 status.) The combo multiplier itself never multiplies light-attack damage (correct per
Melee_Combo.wiki L13). With the option off, light attacks are evaluated at a 12x combo for these bonuses (quirk Q12).

Other combo-like constants (no timeline): sniper combo `J = sniperCombo attack ? (option val or 1.5) : 1` (L10326-10330);
exalted ability combo `Ua` (L10377-10388). Both multiply `Jb` for every hit.

---

## 5. Status timeline (what the aggregation consumes)

`timelineStatus(hits)` (L10093-10136):
```
for each hit k, for each status s in hit.statuses with a statusDuration entry:
    end = hit.time + statusDuration[S].curr_duration
    j   = first index > k... (binary search over hit times) with time > end
    diff[s][k] += 1; diff[s][j] -= 1
stacks_s(i) = min(prefixSum(diff[s])(i), cap_s),   cap_s = max(min(statusDuration[S].max (+incMaxStacks_Corrosion for corrosive),
                                                          maxCountProc || INF), 1)
timeline[i][s] = {ticks: stacks}   (only when > 0)
```
A hit sees its own procs and the procs of earlier pellets of the same shot (index order), not later pellets of the same
shot. Each `getModelQueueDmg` call gets a deep copy of the timeline (L12558 etc.), which it extends with procs created during
the damage pass via `extendTimeline(timeline, i, statuses, hitsPerSec, N, extraMax, hits?)` (L10295-10310):
- time mode (hits passed): stack added to every later hit with `time <= t_i + duration`;
- index mode (no hits passed; Debilitate L10636, Toxic Lash L11497/L11678/L11879): added to the next
  `duration * hitsPerSec` hit indices, `hitsPerSec = round(N / T)` (L10349). With fewer than 0.5 hits/s, `hitsPerSec = 0`
  and these procs never reach the timeline (quirk Q9).
Sagek passive mode skips timelineStatus and builds the timeline incrementally (L12534-12546, L10594-10620).

---

## 6. Per-hit record and `totalDmg`

`getModelQueueDmg(hits, timeline, baseDmg, layer, attackIdx, firstLast, statusMasks, shotSchedule, keyW, keyX, rngMemo)`
(L10311-12014) writes `arrInfo[keyX][attackName][i]` for every hit i (L10572, L10588). Fields relevant to aggregation:

| field | meaning | line |
|---|---|---|
| `timeM` | hit time | L10591 |
| `shotInfo`, `reloadsInfo` | scheduler records (for the "More details" popup) | L10590-10593 |
| `crit`, `statuses`, `meleeMult`, `comboMult` | rolled crit tier multiplier, statuses of this hit, stance mult, combo mult | L10855-10858 |
| `curr_dmg` | direct hit damage on this layer | L11514-11960, x`hc` L11962 |
| `status[s].d` / `.nc` | one-tick damage of each status (crit / non-crit base) | L10895-11360 |
| `xataTotal`, `toxicLash`, `debilitateDmg`, `dmgOnStatusEff` | extra damage sources | L11407, L11496, L11987, L11995 |
| `otherDmg` | Kuva Ogris napalm (`jc`,`ac`) + Reconifex heat (`mc`) | L11997 |
| `totalDmg` | everything credited to this hit | L11998-12010 |

```
totalDmg = ( hb                    // direct + Σ_status (duration_s × tickDmg_s) (Bb), L11958 etc.
           + xata                  // db
           + toxicLash.direct_hit + toxicLash.tick_dmg × ToxicLash.curr_duration
           + debilitate            // Σ tickDmg × duration
           + napalmA + napalmB     // × min(6, remainingWholeSeconds) (L10772-10774)
           + reconifexHeat × Heat.curr_duration
           + dmgOnStatusEff ) × hc // hc = unique-enemy DR factor (1 unless xb), L10557
```
Key property: **all damage over time is front-loaded** at the hit that applies the proc: a Slash proc on the last hit of the
window is credited its full duration even though its ticks would land after T (Bb at L10982-10986, L11287-11289 etc.).
Exception: napalm (Kuva Ogris "Rocket Explosion" with `nNapalm`) is truncated by `remaining = floor(T - i/(N/T))` capped at
6 s (L10772-10774), where the remaining time is computed from the hit index, not its time (quirk Q8). The function returns
`Σ totalDmg` (L12011-12013), which createDmgQueue ignores.

---

## 7. Aggregation and every number shown

### 7.1 Layers computed (createDmgQueue L12550-12693)
For each attack k: `health` always; `healthFull` if the enemy has shields; `armor` (and `armorFull` if shields) if
`currEnemy.currArmor > 0`; `shield` if `currShield > 0`; `overguard` if `currOverguard > 0`. All layer calls of one attack
share the same hit list, the same status rolls and the same RNG memo `t` (L12551), so crit rolls made inside
getModelQueueDmg agree across layers (memo key `hitId:label:index`, L10313-10318). `healthFull` and `armorFull` feed only the
TTK (L12697-12714); the DPS rows use `health`, `armor`, `shield`, `overguard`.

### 7.2 Per-second buckets: `getDmgPerSecond(hits)` (L12787-12801)
```
N = len(hits); if N == 0: return [0]*T
slot = T / N
bucket[s] = Σ_p totalDmg_p × overlap([p*slot, (p+1)*slot), [s, s+1)) / slot     for s in 0..T-1
```
Hits are laid out **by index**, evenly, not at their timestamps (`timeM` is ignored), so reload gaps, burst clumps,
charge delays and the uneven melee multishot copies are smoothed away. `Σ bucket = Σ totalDmg`, so the bucket mean equals
`(total damage of all hits in the window) / T` — a sustained figure that already includes reload downtime through the shot
count.

### 7.3 "Random queue dps" = median of buckets
`getMedianFromTotalQueueDmg(hits) = median(getDmgPerSecond(hits))` (L12802-12805; the second argument is ignored).
`median` (L13269-13273): sort; odd length -> middle; even -> mean of the two middle values (T = 20 -> mean of 10th and 11th).
Stored as `currWeapon.info[attack].queue_<hp|armor|shield|overguard>` (L12595, L12634, L12660, L12686) and pushed rounded into
`dpsQueue[attack][hp|armor|shield|overguard]` (L12596-12693). The changelog confirms the switch from the arithmetic mean to the
median (index.txt L249).

### 7.4 Values in each result row: `createDamageInfoHTML(layerKey, attack, avg, medianPerHit, ttk)` (L12817-12844)

| label | value | derivation |
|---|---|---|
| "Random queue dps" | `parseInt(queue_<layer>)` | §7.3 |
| "avg per bullet (without status dmg)" | `round(avg.direct_dmg)` | `getAvgQueueVal` L12217-12232: `mean_i(curr_dmg + xataTotal + dmgOnStatusEff + toxicLash.direct_hit)` |
| "avg per bullet (status dmg)" | `round(avg.status_dmg)` | `mean_i(getStatusDamageSum(hit_i))`; `getStatusDamageSum` L12205-12216 = `Σ_s value_s × curr_duration_s × (count of s in hit.statuses)` + Toxic Lash `tick_dmg × duration`, where `value = (hit.isNC && d != nc) ? nc : d` (L12197-12204) |
| "avg per bullet (total)" | `round(direct + status)` | sum of the two above |
| "median per bullet (total)" | `round(median_i(totalDmg_i))` | `getMedianQueueVal` L12233-12238 |
| "TTK" | `ttk.toFixed(3) + "s"` or "not killed" | `calcTTKWithStatusUnified([og, shield, health, healthFull], T, pools, statusDuration, hasToxin)` L12705-12715 (L13593 onward); one value shared by the health, shield and overguard rows, a separate one for armor |
| "More details" | popup listing every hit and reload in time order | L14957-15040; legend in index.txt L440-461 ("Legend:" L442) |

"bullet" means one hit entry (one pellet, one beam tick or one melee hit), not one trigger pull. `avg total` and
`mean(totalDmg)` differ: the avg excludes napalm, Reconifex heat and Debilitate, and the status-count weighting differs
(quirk Q10). Row presence: reCalcDisplay creates armor/shield rows from `currEnemy.armor.val` / `shield.val` (L9143-9146) but
createDmgQueue fills them only when `currArmor` / `currShield > 0`.

### 7.5 Multiple attacks of one weapon
Every entry of `currWeapon.attacks` gets its own queue, its own row and its own DPS (loop L12426, L12550). Nothing sums
attacks (no consumer of `queue_*` other than the optimizer's per-attack read at L15777). The guide states attacks "are not
aware of each other's existence" (index.txt L152-155). Attack-specific quirks: e.g. Catabolyst's "Partial Reload" /
"Reload From Empty" attacks are simulated as continuous 12/s streams with the weapon's magazine (sample
`weapon-Catabolyst.json`), and because the weapon is tagged BEAM each costs 0.5 ammo (quirk Q5).

---

## 8. "Combine attack effects" (L12407-12413, L12540-12553; help text index.txt L433-438)

Weapons carry `comb` groups of attack indices (Acceltra Prime `[[0,1]]`, Lenz `[[0,1,2]]`, Catabolyst `[[1,2],[3,4]]`).
Each group has a checkbox `.combCxb` (built at L7998). For a checked group:
```
for each attack x in the group, for each hit index H of x:  combined[H].statuses += hits_x[H].statuses     (L12540-12545)
combinedTimeline = timelineStatus(combined)                                                                (L12549)
each attack in the group uses combinedTimeline instead of its own                                          (L12553)
```
Only the status timeline (debuffs: Viral, Corrosive, Heat armor strip, Cold crit, Magnetic, stack counts for CO) is shared;
direct and status damage are still per attack and not summed. Hits are paired **by index**, not time. The combined entries
have no `time` field, so `end = undefined + duration = NaN`, the binary search runs to the end, and **every proc in a combined
timeline lasts until the end of the window** (stacks only grow to the cap) — quirk Q4.

---

## 9. "Calculate average values over several full firing/attack cycles" (`.butNewQueue`, L14273-14485; HTML L387)

```
clear dpsQueue, TTK_Queue, MedianTotal_Queue, TotalDmg_Queue; isQueueRun = true
z = setInterval(reCalc, 0)                           // re-runs the whole calculation as fast as the browser allows
after 5000 ms: clearInterval; for each attack and layer:
    "Random queue dps" text = parseInt(median(dpsQueue[attack][layer]))       (per-run DPS values, rounded)
    tips: Min = min(dpsQueue), "Avg" = round(median(dpsQueue)), Max = max(dpsQueue)
    TTK  = median(TTK_Queue with null -> Infinity)  -> "not killed" if Infinity
    "median per bullet (total)" = median(MedianTotal_Queue)
    "avg per bullet (total)"    = median(TotalDmg_Queue)     // per-run avg totals, rounded
isQueueRun = false
```
So "averaging" is the **median of per-run medians over as many full 20 s queues as fit in 5 wall-clock seconds** (a
CPU-dependent sample count). The tip labelled "Avg" is a median. "avg per bullet" direct/status values are not updated (they
keep the last run's values). Each run uses the same `queueLength`, so "several full cycles" means several random repetitions
of the same window, not a longer window.

## 10. Window length, burst vs sustained DPS

- `queueLength` default 20 s (L5496); slider `#queueLen` 1-60 s, step 1, label "Continuous firing / attack duration (sec)"
  (HTML index.html L387-393, JS L14863-14871); any change triggers `reCalc`. The optimizer runs with its own window `Aa.queueLength`
  (default 20) and restores the global afterwards (L18884-18941, L19037-19150, L19199-19384).
- There is **no separate burst-DPS or sustained-DPS output**. The single number is "damage in the first T seconds of
  continuous fire from a full magazine / zero combo state, divided into seconds, median". It behaves as burst DPS when T is
  shorter than one magazine and approaches sustained DPS for long T; for intermediate T it depends on the window phase
  (Furis base: 144 shots in 20 s = 7.2 shots/s vs the analytic sustained `35/(3.5+1.4) = 7.14`).

---

## 11. Randomness sources and what an expected-value engine needs

Random draws (all `Math.random`, directly or via `isProc`):
1. Pellet count per shot (L10059) and beam `multishotTier` (L10007).
2. Number of statuses per hit and status type per proc (procStatusArray L10011-10043; type via `floor(100*rand)` into a
   100-slot mask built by `makeRNDMaskStatusArray`, L12239-12246) — detail in `inspiration-status-and-procs.md`.
3. Random-status mods, Cedo elements, impact-triggered Slash/Magnetic (setForceProcs L12845-12878, setProcsITS L9853-9878).
4. Inside getModelQueueDmg, memoised per `hitId:label:index` (L10313-10318): crit tier, double crit, Hunter Munitions,
   Gotva crit-after-status, Sagek, Debilitate element, Melee Duplicate (L10594-10766).
5. Fractional chances are rounded to whole percent before rolling (`Math.round(frac*100)`, L10054, L10679, L10005, L10012).

Not random: shot/hit times, reloads, magazine, ammo efficiency (expected multiplier), melee hit counts, stance multipliers,
combo timeline.

To reproduce the inspiration's numbers deterministically, an engine must replicate:
- the exact scheduler of §2 (including its quirks) and the melee counts of §4, since shot counts in the window drive DPS;
- expected per-hit damage, which is **not** the product of expected factors, because debuff stacks (Viral, Corrosive,
  Magnetic, CO status count, Cold) depend on the random proc history inside a moving duration window and are capped; an EV
  engine needs the expected stack-count distribution per status over time (or per-hit `E[f(stacks)]`), not just `E[stacks]`;
- full-duration DoT crediting at proc time (§6) rather than window-truncated ticks;
- the 1 %-quantised fractional chances if bit-matching, or exact fractions if matching the game;
- the final statistic: the inspiration shows a **median** of per-second sums. Its expected value is not `E[total]/T`; for
  low hit rates and high crit variance the per-second sums are right-skewed and the median sits below the mean. A
  deterministic "mean DPS" will therefore read higher than the inspiration for slow, crit-heavy weapons, and roughly equal for
  fast weapons. Matching the displayed figure exactly needs Monte Carlo with the same bucketing and median; the averaging
  button converges to the median of the per-run median distribution, which is also not the mean.

The optimizer's own "fast estimate" (stratification) replaces `isProc` with an error-diffusion accumulator per
`(chance, denominator)` key seeded by a hash (`Ca`, L15688-15704) and `Math.random` with a golden-ratio Weyl sequence
`x = (x + 0.6180339887498949) mod 1` (`ta`, L15705-15707; constant L21826; install/restore L15708-15725). Its own help text
says this breaks when several conditions multiply (L15988), and it validates against "honest" stochastic runs
(L15966-15978, L18902, L19046, L19206).

---

## Suspected inspiration quirks

- **Q1 Bow double reload.** Bows get `Ca = 1/(charge+reload)` and, with magazine 1, a magazine reload after every shot, so the
  shot interval is `charge + 2*reload` (Lenz-like: 2.4 s instead of 1.8 s; 8 shots in 20 s instead of 11). L12290-12293 with
  L12333-12348, L12381. Wiki: `1/(charge + reload)` (Fire_Rate.wiki L39-45).
- **Q2 Non-bow charge weapons use reload instead of fire rate.** `Ca = 1/(charge + reload)` per shot plus magazine reloads
  (L12296). Wiki: `1/(charge + 1/fireRate)` (Fire_Rate.wiki L30-37). Also only shot 0 adds the charge explicitly (L12351).
- **Q3 Shot-count bound truncation.** `Pa` (L12311-12321) undercounts when reload < 1/Ca (one shot lost per partial
  cycle) and is 0 when `T*Ca < 1` (rate below 1/T) even though a shot at t=0 is possible; such weapons show 0 DPS for short
  windows.
- **Q4 Combined timelines never expire.** Combined status entries lack `time` (L12543-12545), so `timelineStatus` makes every
  proc last to the end of the window (L10103-10109) and pairs attacks by index.
- **Q5 Beam ammo for every attack of a BEAM-tagged weapon.** Cost 0.5 applies to all attacks of a weapon whose compTags include
  BEAM (L12268-12273, L12474-12478), e.g. Catabolyst reload grenades; Flamethrower exception (Continuous_Weapon.wiki L145) not
  modelled; beam damage ramp-up not modelled.
- **Q6 Battery recharge uses the ammo-efficiency-inflated magazine** (`mag*effMult / reloadRate`, L12278 then L12286), and
  `currWeapon.reloadRate = base / (1 + reloadRate mods)` (L9019-9023) divides a rate; also custom reload stat is omitted there.
- **Q7 Burst delay modding multiplies by `(1 + speed_mult)`** instead of dividing (L8941-8952), so `speed_mult` lengthens the
  burst delay.
- **Q8 Napalm truncation by index time.** Remaining seconds use `T - i/(N/T)` (L10772), not the hit time, and only napalm is
  truncated; all other DoT is credited in full past the window end (§6).
- **Q9 Index-mode status extension** uses `round(N/T)` hits per second (L10349, L10305): wrong for bursty schedules and zero
  for < 0.5 hits/s.
- **Q10 Per-bullet averages are inconsistent with totalDmg** (L12217-12232 vs L11998-12010): avg excludes napalm,
  Reconifex heat, Debilitate; Xata gets `hc` twice in totalDmg (L11404 then L12009). The optimizer's `freshHit` adds
  `getStatusDamageSum` on top of `totalDmg`, which already includes status DoT (L15737-15740) — looks like double counting.
- **Q11 Slam attacks receive `statuses[0]` of the chosen stance list on every hit** (L12142-12148), e.g. Nikana Heavy Slam gets
  a forced Slash each hit; slam hit rate is 1/s regardless of attack speed without a stance mod but `1/speed` with one
  (L10186 vs L10201-10209).
- **Q12 Light attacks assume 12x combo by default** for per-combo crit/status bonuses (`combo_counter = 11`, L8219-8224,
  L9906-9910).
- **Q13 Heavy combo model is decay-only**: combo is spent every `int(0.8*N/T) + heavyHits` hits (L12169-12180), never rebuilt by
  hits; Initial Combo is regained instantly instead of at 40/s (Melee_Combo.wiki L135).
- **Q14 Melee timing is uniform** (L12445-12446) and heavy hit count always uses the default stance table, even with a stance
  mod (L10204).
- **Q15 Per-second bucketing ignores timestamps** (L12789-12797): reload gaps and bursts do not affect the per-second spread.
- **Q16 First/last marks follow shot index groups of `perMag`** even for infinite magazines (L12472-12481 vs L12298) and mark
  the final partial group's last shot as "last" though the magazine is not empty.
- **Q17 Cosmetic:** `shotIndex` 0 is falsy, so hit 0 has `shotInfo = null` (L10590); the "Avg" tip after averaging is a
  median (L14327-14328); averaging sample count depends on CPU speed (L14286, L14291-14483).
- **Q18 Shot exactly at T is counted** when reached through the loop (`oa > T` break, L12353) but excluded by `floor(T*Ca)` in
  the no-reload branch, so boundary inclusion differs by branch.

## Open questions

- Rationale of the `0.8 * hits / T` term in addTimelineMeleeCombo (L12169): whether it models time between heavy attacks or is
  an empirical fudge is not documented anywhere in the bundle or guide.
- getMeleeCountHits10s: an attack with a `type` other than "h"/"s"/"hs" on a weapon in the default stance table leaves
  `k`/`m` undefined (L10155-10186) and would throw; no sample weapon has such a type, so whether this path occurs is unknown.
- Whether `reloadRate` in battery weapon data is ammo per second or seconds per ammo (determines whether L9019-9023 is a bug).
- How `health` differs from `healthFull` (and `armor` from `armorFull`) in `getTotalTypeOfArmorDmg` — out of scope here, needed
  to explain why DPS uses `health` but TTK uses both.
- Whether a status applied by a hit should affect that same hit's damage in game (inspiration: yes, own procs and earlier
  pellets of the same shot count, L10111). Cross-check with `wiki-damage-types-and-status-effects.md`.
- The game's real reload start (immediately after the last shot vs after one more fire interval, L12381) is not confirmed by
  the wiki pages read.
