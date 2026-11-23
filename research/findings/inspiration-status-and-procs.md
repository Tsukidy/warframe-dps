Status: draft

# Inspiration: status effects, proc rolling and per-hit damage application

Source: `research/external/inspiration/min.pretty.js` (abbreviated "L" + line). Function names are the bundle's own; local variable names are minified, so each formula below is rewritten with readable names and the line range it comes from. Wiki comparisons cite `research/findings/wiki-damage-types-and-status-effects.md` ("wiki-status §x") or the cached wiki page in `research/external/cache/wiki/`.

Scope: L9779-10311 (proc helpers), L10311-12015 (`getModelQueueDmg`, read only where it applies statuses), L12015-12250, L12250-12550 (`createDmgQueue` pipeline order), L12845-12884, L13193-13252, plus `getTickStatusDmg` (L9154-9204), the status table (L5544-5580) and the crit helpers (L8145-8329) that the proc code calls.

## 1. Big picture

1. The DPS is a Monte-Carlo "queue" of `queueLength = 20` seconds (L5496). `createDmgQueue` (L12250) builds a list of hit objects `{crit, statuses[], time, shotIndex}`: one per pellet per shot (L12509-12516) or, for melee, per stance hit (L12436-12457).
2. Statuses are rolled **once per hit, before damage is computed**, in this order (L12526-12547):
   1. melee only: stance forced procs pushed as strings (`procMeleeComboAttackEffect`, L12115-12150; L12527-12531);
   2. `makeRNDMaskStatusArray` builds the 100-slot type mask (L12532);
   3. `procStatusArray` rolls the status-chance procs (L12533);
   4. `setForceProcs` adds weapon forced procs (L12536);
   5. `setProcsITS` adds Internal Bleeding / Hemorrhage / Magnetic Welt procs (L12537);
   6. optional "Combine attack effects": statuses of attack hits with the same **index** are merged (L12540-12545, L12549);
   7. `timelineStatus` turns the per-hit statuses into a per-hit "stacks on target" map (L12546).
3. `getModelQueueDmg` (L10311) is then run once per defence layer (health, healthFull, armor, armorFull, shield, overguard; L12559-12682). It reads the stacks map to apply debuffs, rolls crit, adds Hunter Munitions, and credits the **entire DoT of every proc to the hit that applied it** (no tick timing in the DPS number; §6).
4. All random draws inside `getModelQueueDmg` go through a memo keyed by `hitId:label:index` (L10313-10318, `E`/`F`) shared across the layer calls of one attack (the same `t` object, L12551), so crit and Hunter Munitions rolls agree between layers. `procStatusArray`, `setForceProcs` and `setProcsITS` use raw `Math.random()` but run only once per attack.

`isProc(chance, of)` = `of > 0 && random()*of < chance` (L9819-9821). Every chance below is "x out of 100".

## 2. Status table (`statusDuration`, L5544-5580)

| Key | duration (s) | `ticks` | `max` stacks | `delay` |
|---|---|---|---|---|
| Impact | 1 | 1 | 5 | 0 |
| Slash | 6 | 6 | Infinity | 1 |
| Puncture | 6 | 0 | 5 | 0 |
| Heat | 6 | 6 | Infinity | 1 |
| Electricity | 6 | 6 | Infinity | 0 |
| Toxin | 6 | 6 | Infinity | 1 |
| ToxicLash | 7.5 | 7.5 | Infinity | 1 |
| Cold | 6 | 0 | 10 | 0 |
| Blast | 1.5 | 0 | Infinity | 1 |
| Corrosive | 8 | 0 | 10 | 0 |
| Gas | 6 | 6 | 10 | 0 |
| Magnetic | 6 | 0 | 10 | 0 |
| Radiation | 12 | 0 | 10 | 0 |
| Viral | 6 | 0 | 10 | 0 |
| Void | 3 | 0 | 1 | 0 |
| Microwave | 1 | 0 | 1 | 0 |
| Tau | 8 | 0 | 10 | 0 |

- **Status duration mods** (`setFinal...`, L9085-9093): `SD = 1 + mods.status_duration + unique.status_duration`; for every key `curr_duration = parseInt(duration × SD)`, i.e. **truncated to whole seconds**. `curr_duration` is used both as the DoT tick count (§6) and as the stack lifetime in the timeline (§4). `ticks` and `delay` are only read by the TTK simulator (L13674-13684, which uses the **unmodded** `duration`/`ticks`).
- `ToxicLash` 7.5 s = 6 s × 1.25: Saryn's passive (+25% status duration) baked in (inference; not commented in code).
- **Enemy cap** `maxCountProc = currEnemy.maxProcStacks ?? 0` (L7711); 0 means none. In `enemies-min.json` 13 enemies have 4 and one has 10. It limits (a) the number of same-type procs **within one hit** (L10024-10031) and (b) the stack count read from the timeline (L10119-10126). It does **not** limit DoT damage (every proc is credited, §6).
- **Immunity** `enemyImmunStatus = currEnemy.immun.status ?? []` (L7710); `"all"` disables every status effect and status damage (`Nb`, L10520, L10883).

## 3. Rolling procs

### 3.1 Type weights: `getCurrTypeChanceForStatus` + `makeRNDMaskStatusArray` (L10063-10077, L12239-12246)

```
statusTotal   = Σ attack.damage[t] for t whose ucFirst(t) is a statusDuration key     (calculateTotalDmg, L10063-10068)
denominator   = statusTotal − Σ attack.damage[t] for t in enemy.immun.status            (L10074-10075)
share(t)      = 0                    if t immune or t has no statusDuration entry      (L10073)
              = 100 × damage[t] / denominator
mask          = for each damage key t in object order: push t, ceil(share(t)) times    (L12241-12244; loop `for q=0; q<p; q++`)
```

- `attack` is `currWeapon.attacks[i]` **after** `newElements` + `reCalcWithTypeBase` (L9638 `reCalc`: L9661-9662), so the weights are modded per-type damage: element combination, elemental mods, physical mods (`getPhysMods`, L9400) and base-damage mods (which cancel). This matches the wiki rule (wiki-status §2.2).
- Immune types are removed and the rest renormalised, matching the wiki.
- A type is drawn as `K = floor(100·random())`, `type = mask[K]` (L10022-10023). Because each type gets `ceil(share)` slots and only slots 0-99 can be drawn, the mask is usually longer than 100 and **types listed last in the damage object lose up to (n−1) percentage points** (quirk Q1).

### 3.2 `getFinalStatusChance(attackIdx, comboMult, extraMult, absExtra, tauStacks, override)` (L9888-9977)

```
if override > 0: return override                                     (L9889; Sagek uses 500)
if attack.noIncrStatus: return attackInitial.status_chance           (L9911, L9976)

base   = mods.set_status_chance  if > 0                              (L9890, L9912-9914)
       = attackInitial.status_chance + 100 × critToStatusBonus        (L9942-9948; inc_flat_status_chance_per_crit_up: min(max, CC·val/100))
flat   = 100 × (incarnon flat_status_chance + attackUnique.flat_status_chance)   (L9896-9899, L9915-9920)
combo  = (comboMult − 1)                 if comboMult > 0 and (heavy or heavy-slam)
       = (meleeComboSetting − 1)         if that setting is on
       = combo_counter (default 11)                                   (L9905-9910)
       × mods.status_chance_by_combo     (Weeping Wounds 0.4, L1720)
add    = 1 + mods.status_chance + extraMult + combo + custom.status_chance
           + (heavy ? mods.h_status_chance : 0) + attackUnique.status_chance
abs    = mods.absolute_status_chance + custom + absExtra + attackUnique.absolute_status_chance
mult   = 1 + mods.status_chance_mult + custom.status_chance_mult + 0.1 × tauStacks   (L9891-9895)

SC% = ((base + flat) × add + abs) × mult                             (L9913-9940 / L9944-9972)
```

- `extraMult` is `globalModStatusChance` for attack 0 only (L10001), a status-chance bonus from a selected weapon option (L9459).
- No clamp: SC above 100% is allowed and produces extra procs.

### 3.3 `procStatusArray(hits, attackIdx, isBeam, multishot, mask)` (L9978-10049)

Per hit `z`:

1. **Tau self-vulnerability** (only if the attack has Tau damage, L9989): keep the times of earlier Tau procs; `tauStacks = min(10, #Tau procs with hitTime − t < Tau.curr_duration)` (L9993-9996); passed to `getFinalStatusChance`, so each stack adds +10% to the `mult` term (max ×2.0). The current hit's own Tau procs are added after the roll (L10044-10046). Matches wiki-status §4.15 (×(1+0.10n)).
2. `SC = getFinalStatusChance(attackIdx, hit.comboMult, ...)` (L10001).
3. **Beams** (`isBeam && multishot > 0`, L10003-10010): `tier = floor(ms) + (isProc(round(frac(ms)·100)) ? 1 : 0)`; `hit.multishotTier = tier`; `SC *= tier`. Non-beams: `multishotTier = 1` (L10002) because each pellet is already its own hit (`procMultishotArray`, L10050-10062). This matches the wiki (Multishot §Continuous Weapons: merged beams sum damage and SC; crit not summed).
4. **Number of procs**: `n = floor(SC/100)` guaranteed procs plus one more with `isProc(round(frac(SC/100)·100))` (L10011-10016). The fractional part is **rounded to a whole percent** (quirk Q2). Placeholders `true` are appended to `hit.statuses` (which may already hold stance strings).
5. **Type of each proc**, iterating placeholders from the end (L10020-10038):
   - `K = floor(100·random())`; if `mask[K]` exists:
     - `limit = maxCountProc`, or `maxCountProc + incMaxStacks_Corrosion` for Corrosive (Emerald shard, L5350-5364) when both are > 0 (L10025-10026);
     - if `limit == 0` or fewer than `limit` procs of that type are already in this hit, the placeholder becomes the type; otherwise it is removed (L10027-10031);
     - Archon Continuity (`corrosive_by_toxin`) counts Toxin procs, Archon Vitality (`archon_vitality`) counts Heat procs (L10029-10030).
   - else (only reachable when the mask is shorter than 100, i.e. every status type is immune → share = NaN) `splice(J, 1 + floor(attack.status_chance/100))` (L10032). Quirk Q3.
   - Secondary Encumber (`random_status_after_status`, 0.24 at L4548, disabled when the weapon has `sagekPassive`): for each placeholder, until one succeeds, roll 24%; on success pick `x[round(random·12)]` from the 13 types Impact..Viral (L9986, L10033-10037). At most one per **hit object** (L10039-10041), subject to `maxCountProc`.
6. After the loop: if ≥1 Toxin proc and Archon Continuity → push **one** `"corrosive"`; if ≥1 Heat proc and Archon Vitality → push **one** extra `"heat"` (L10042-10043).

Duplicates are allowed (independent draws), matching wiki-status §2.1.

### 3.4 Forced procs

- **`setForceProcs(hits, attack)`** (L12845-12877), applied to every hit of the attack:
  - `attack.unique.force_procs`, or `attack.unique.WITH_COND.force_procs` when "all conditions" is on (`isMaxCond`, default true, L5460) (L12846-12850, L12869-12872);
  - Evensong: +7 `"puncture"` on every hit (L12851-12853);
  - Cedo / Cedo Prime attacks with `cedoRnd`: one random base element + one random damage type of the attack (L12854-12863);
  - each weapon forced proc also rolls Encumber (24%) for one random extra type (L12873-12876).
- **Stance forced procs** (`procMeleeComboAttackEffect`, L12142-12148): the stance's `statuses[p]` string for each hit, cycling with `hits[p]`; also stored as `stanceStatuses`.
- Forced procs are added **after** the per-hit `maxCountProc` check and are not limited by it (only by the timeline cap, §4).

### 3.5 Internal Bleeding / Hemorrhage / Magnetic Welt: `setProcsITS(hits, fireRate)` (L9853-9878)

```
chance = 100 × mods.add_slash_on_impact            (Internal Bleeding / Hemorrhage 0.35, L1765, L1771)
if fireRate < 2.5 and weapon is not melee: chance ×= 2   (L9858)
for each hit whose statuses include "impact" (any number of Impact procs): if isProc(chance): push "slash", hit.hasITS = true
same for add_magnetic_on_impact (Magnetic Welt 0.35, L1777) → push "magnetic"
```

`fireRate` = `currWeapon.attacks[i].speed` (L12537). Once per hit even with several Impact procs: matches Internal_Bleeding.wiki ("Proccing impact more than once ... will not allow this mod to proc more than once"). These procs are added before `timelineStatus`, so they count as stacks.

### 3.6 Hunter Munitions (L9879-9881, L10761-10766)

```
hasHM = any mod named "Hunter Munitions"            (its table entry add_slash: 0.3, L1759, is ignored; 0.3 is hard-coded)
if hasHM and not hit.hasITS:
    cc = min(finalCritChance%, 100)
    if (hit crit multiplier > 1  or cc == 100) and isProc(0.3 × cc): statuses.push("slash")
```

- Rolled inside `getModelQueueDmg`, after the crit roll, and only into the local `statuses` copy (`ca`): the Slash DoT is credited but the proc is **not** added to the timeline (no `extendTimeline`), so it never counts as a stack.
- Skipped when Internal Bleeding already fired on the hit. Wiki (Hunter_Munitions.wiki L40) says both roll independently and only one Slash results, so the outcome matches.
- **Bug (Q4):** the game rolls 30% per crit (Hunter_Munitions.wiki L38). The inspiration requires a crit *and* a `0.3 × CC` roll, so P(HM per hit) = CC × 0.3 × CC instead of CC × 0.3 (e.g. CC 50%: 7.5% instead of 15%). Correct only at CC ≥ 100%.

### 3.7 Other proc sources inside `getModelQueueDmg`

| Source | Lines | Behaviour |
|---|---|---|
| Sagek passive (`sagekPassive`) | L10594-10620, L10694, L12534-12535 | After a crit, chance `sagekPassive` that the **next** hit replaces its rolled procs with a 500%-SC roll (`setStatusesToHit`, no forced procs), keeping forced procs; Encumber rolls once per distinct hit time here (L10611-10616). Added to the timeline with `extendTimeline`. |
| Primary Debilitate (`debilitate`, L4616) | L10622-10639 | If the hit applied a combined element that has ≥10 stacks, pick one such element at random, then one of its two components (50/50), and add one proc of it to the timeline. Damage in §6.4. |
| Secondary Cryogenic (`onPunctureApplyCold`: 3, L5190) | L10742-10751 | If the hit has a Puncture proc and the enemy is not immune to Cold or Puncture, add 3 Cold procs (statuses + timeline). |
| `addPunctureStatus` | L10641-10647 | Adds N Puncture stacks to **this hit's** timeline entry only (capped at 5). |
| Melee Duplicate (`mDuplicate`, L4740) | L10697-10741 | Every yellow (tier 1) crit of a non-slam attack inserts a duplicate hit right after it. The duplicate re-rolls crit (in its own loop iteration) and status with `setStatusesToHit(attack.status_chance)` (the displayed modded SC, not `getFinalStatusChance`) plus the original stance procs. Its timeline entry is a copy with +1 per new status capped at `statusDuration.max` (not `maxCountProc`); later hits do not see its procs. Assumes rank 5 (100%), matching Melee_Duplicate.wiki. |
| Toxic Lash (option) | L11463-11497, L11823-11880 | Always adds one Toxin stack (timeline) per hit; damage in §6.5. |

`setStatusesToHit(attackIdx, SC%, masks, contByToxin, vitality, rng, label, withForced=true)` (L10254-10282): same algorithm as §3.3 steps 4-6 but types not found in the mask are simply dropped, no per-hit cap, no Tau, then appends `force_procs` unless `withForced=false`.

## 4. Stacks on the target: `timelineStatus` and `extendTimeline`

### 4.1 `timelineStatus(hits)` (L10093-10136)

```
for each hit k (time t_k), for each status s in hit.statuses with a statusDuration entry:
    end = t_k + curr_duration(s)
    w   = first index ≥ k with hits[w].time > end        (binary search, L10105-10109)
    diff[s][k] += 1 ; diff[s][w] −= 1                     (L10110-10112)
cap(s) = max(1, min(statusDuration[s].max (+incMaxStacks_Corrosion for Corrosive),
                    maxCountProc (+incMaxStacks_Corrosion for Corrosive) or ∞))   (L10119-10127)
stacks[k][s] = min(prefixSum(diff[s])[k], cap(s))         (L10134)
```

- A proc counts from the hit that applied it (inclusive) until the last hit whose time is ≤ `t + curr_duration`. Hits are ordered by array index, not re-sorted by time.
- Capping the count of live procs at `max` is equivalent to "new stack replaces the oldest" for capped types, which matches the wiki for Corrosive, Viral, Magnetic, Gas, Radiation, Tau, Cold (wiki-status §1 table).
- **Quirk Q5 (own-hit and same-shot timing):** the applying hit already sees its own stack (index k is inclusive), and so do later pellets of the same shot (same time, higher index), while earlier pellets of that shot do not. The wiki says a hit is damaged before its own Viral proc lands and pellets of one shot do not benefit from each other (Viral page, wiki-status §4.13). This overstates Viral/Magnetic/Corrosive/Cold/Puncture effects for the hit that applies them.
- Hunter Munitions procs are not in the timeline (§3.6).
- The Sagek path skips `timelineStatus` and fills the timeline incrementally with `extendTimeline` (L12546, L10619).

### 4.2 `extendTimeline(timeline, idx, statuses, hitsPerSec, nHits, extraMax, hits?)` (L10295-10310) and `addTick` (L10078-10092)

- With `hits` given: adds +1 stack of each status to every entry from `idx` while `hits[E].time ≤ hits[idx].time + curr_duration`.
- Without `hits`: uses an index estimate, entries `idx .. idx + curr_duration × hitsPerSec − 1`, where `hitsPerSec = round(nHits / 20)` (L10349).
- `addTick` increments up to `min(max (+Corrosive extra), maxCountProc (+Corrosive extra) or ∞)`. Note `extendTimeline` without `hits` mutates the shared entry objects (the array copy is shallow).

## 5. Debuffs applied per hit (inside `getModelQueueDmg`)

`stacks = d[A]` is the timeline entry of the current hit. Layer flags: `isArmor`, `isHealth`, `isShield`, `isOG` (L10333-10336).

### 5.1 Viral (L10779-10801)
- Active on health and armor layers (`Kd`: not shield, not OG, not immune to viral/all; L10476-10482).
- `viralMult = 2 + 0.25 × (stacks − 1)` (1 stack ×2.0, 10 stacks ×4.25). Matches wiki-status §4.13.
- External Viral option (`externalVirus.stucks`): adds to the stack count, capped at 10 (and at `maxCountProc` only when it exceeded 10, L10784-10788); with no Viral procs it uses the option value uncapped (L10793-10797).
- Multiplies direct damage (`Jb`, L10860-10880), every DoT (`Ya`), napalm (`jc`, `ac`) and Cascadia Empowered.
- **Bug Q6 (sticky multiplier):** `viralMult` (`I`) and `magneticMult` (`Q`) are declared once per call (L10320-10321) and only reassigned when stacks exist, so after the last Viral/Magnetic stack expires the **old multiplier stays** for the rest of the queue.

### 5.2 Magnetic (L10802-10804)
- Active on shield and OG layers only (`Vd`, L10483-10488). `magMult = 2 + 0.25 × (stacks − 1)`. Matches wiki-status §4.11.
- The shield/OG-break forced Electricity proc (wiki-status §4.11) is **not modelled** (no code path).

### 5.3 Armor strip (L10805-10820), armor layer only

```
armorMult = 1
if mods.decr_armor_by_punc > 0 and Puncture stacks and not immune:  armorMult ×= 1 − decr_armor_by_punc × punctureStacks
if Corrosive stacks and not immune:                                 armorMult ×= 1 − (0.20 + 0.06 × corrosiveStacks)
if (Heat stacks or Nightwatch Napalm on Rocket Explosion or reconifexHeat > 0) and not heat-immune:  armorMult ×= 0.5
if mods.armor > 0 (e.g. Corrosive Projection-style aura stat):       armorMult ×= 1 − mods.armor
armorMult = max(armorMult, 0)
effectiveArmor = currEnemyArmorVal × armorMult
```

- Corrosive: 26% at 1 stack, 80% at 10, 92%/98% with the Emerald shard's +2/+3 stacks (via the raised cap). Matches wiki-status §4.9.
- Heat: a flat 50% the moment any Heat stack exists. No 2 s ramp and no 6 s ramp back (wiki-status §4.4). Simplification Q7.
- All sources multiply, which matches the wiki form `Base × (1−0.5h) × (1−(0.2+0.06n)) × (1−0.18·CP)`.
- The **"Decrease armor" option** (`decreaseArmorOpt`) is not part of `armorMult`. It reduces base armor when the enemy is loaded (L7698, L7707) and is re-applied to the caps when armor is scaled: `if armor > 2700: armor = 2700 × (1 − d)`; `if 0 < armor ≤ 200: armor = 200 × (1 − d)` (L8114-8115). So the status strips multiply the already capped value. The ≤200 rule raises small armor values to 200; see Q8.
- Damage reduction: `DR(a) = sqrt(3a)/100` (`getDamageReduction`, L8148-8150); `getNetArmor` is the identity (L8145-8147). No clamp inside the function (armor is capped at 2700 → DR ≤ 0.9).
- `armorDmgWithDecreaseArmor(armorMult, perTypeDamage)` (L9801-9818): per attack and type, `dmg × (1 − DR(armor × max(armorMult, 0))) × (1 − innateDR.armor)`, Finisher damage unreduced, results in (0, 1) raised to 1. Returns `{}` when armor is 0. Used for the direct hit on the armor layer (L11364-11375).

### 5.4 Cold (L10821-10825)
```
if Cold stacks and not cold-immune and hit crit multiplier > 1:
    critMult += 0.05 × coldStacks ; if coldStacks == 10: critMult += 0.5
```
- Adds to the **final** per-hit multiplier, after tier, headshot and the ×2 headshot-crit term (§5.7), so the bonus is never multiplied by the tier. Not excluded for AoE.
- Wiki (Status_Effect.wiki L58; wiki-status §4.5): +0.10 at 1 stack, +0.05 per extra, +0.50 at 9; Frozen (10th) +1.0; absolute CD added **before** the tier is computed; not for AoE. Inspiration is 0.05 low at 1-9 stacks, right at 10, and undervalues it for orange/red crits and headshots (Q9). Frozen state, 3 remaining stacks after thaw and the boss cap of 4 are not modelled (the cap comes only from `maxCountProc`).

### 5.5 Puncture (L10640-10672)
- Crit chance: `getFinalWeaponCritChance(attack, i, 0.05 × punctureStacks + rampingBonus, ...)` → added to the flat crit-chance term (`g`, L8231-8239), which is then multiplied by the `mult_crit_chance` factor (L8266). Not applied to AoE attacks (`Kb`) or if immune. Max 5 stacks → +25%. Matches wiki-status §4.2.
- Duration is 6 s in the table versus 10 s on the wiki (Q10). The enemy-damage reduction is irrelevant for DPS and not modelled.

### 5.6 Tau (L9987-9996, L10044-10046)
SC multiplier +10% per stack, max 10 (§3.3). Tau stacks are tracked inside `procStatusArray` per attack, separately from the timeline.

### 5.7 How the crit multiplier is set: `setCritDmgToHit` + `getFinalWeaponCritMulti`

`getModelQueueDmg` rolls `critRolled = isProc(round(frac(CC/100)·100))` (L10679, again rounded to whole percent) and `doubleCrit = isProc(100 × (mods.double_crit + unique.double_crit))` (L10680-10683), then:

`setCritDmgToHit(attack, critRolled, CC, headshot, i, doubleCrit)` (L9822-9852):
```
if CC ≥ 100:
    if not critRolled: tierInput = CC − 99          (L9826, L9833-9840)
    else:              tierInput = CC
elif critRolled:       tierInput = CC
else:                  return {val: 1, tier: 0}
return getFinalWeaponCritMulti(attack, tierInput, headshot && !attack.no_headshot_mult, doubleCrit, i)
```
`getFinalWeaponCritMulti(attack, cc, isHS, doubleCrit, i)` (L8268-8329):
```
CD     = (attack.crit_mult + flat_critmult [+ flat_critmult_below_cchance if cc < cond]) × (1 + mods.crit_mult + custom + attackUnique + heavy_crit_mult)
add    = mods.crit_mult_add + attackUnique + custom
punc   = min(50, puncShare × attack.status_chance × 0.1)  if incr_CM_by_punc_status > 0    (L8302-8311)
mm     = 1 + custom.crit_mult_mult
if doubleCrit: cc += 100
tier   = 1 if cc ≤ 100 else ceil(cc/100)            (L8314, L8321-8322)
val    = (1 + tier × (CD − 1) + add + punc) × mm                          (no headshot)
       = (HS × (1 + tier × (2·CD − 1)) + add + punc) × mm                 (headshot crit; HS = headShotMult, default 3)
```
- Tier: `CC − 99` on a failed roll emulates `floor(CC/100)` for CC in [100, 199], [200, 299] etc., **except** when `CC mod 100 > 99` (e.g. CC 199.5% failed → `ceil(100.5/100) = 2` instead of 1). Q11.
- Headshot on a non-crit: `Ma = headShotMult` when `crit == 1`, else 1 (L10777), because a crit's `val` already contains HS.
- After this, the hit's multiplier is modified by: disableBodyCrit (L10693), Gotva `crit_after_status` (L10674-10678, sets CC), `flat_crit_with_status` (×(1+x) when total live stacks < 3, L10752-10760), Cold (§5.4), stealth (L10826).
- `procCritArray(n, CC)` (L9882-9887) returns `n` independent rolls of the fractional CC; it is defined but has no call site in the bundle (grep finds only the definition): dead code.

## 6. Status damage

### 6.1 Shared inputs (per hit, L10393-10502, L10827-10852)

```
faction     = getFactionMod()           = factionMod (+ Roar-like rhinoBuff)                       (L9104-9109)
baseDmg     (kb) = Σ_type baseDamage[type] × perTypeBaseMult × faction × dblMult
    with conditions on (isMaxCond) and the attack not AoE and not a slam (L10827-10852):
        additive CO (default):        perTypeBaseMult = statusCount × base_per_status + base_per_cold + baseMods × sharpshot
        multiplicative CO (isCoMult attack flag or the "multiplicative" option):
                                      perTypeBaseMult = (1 + base_per_cold + statusCount × base_per_status) × baseMods × sharpshot
        flat_change_dmg (additive only): Σ = flat × baseMods + (Σbase − flat) × (statusCount·bps + bpc + baseMods·sharpshot)
    otherwise: kb = getModdedBaseDmg(attack) × faction  (× sharpshot for AoE)   (L10393, L10405)
ncBaseDmg   (fa) = same with BaseModsDmg(true) (non-crit "base_noncrit" bonuses, L8462-8466)
statusDmg   (zb) = (1 + status_damage + custom + heavy ? h_status_damage) × (1 + multiple) × (1 + dmgMultAgainstFrozen) × (1 + vuln_status_damage)   (L10498-10502)
hitMults    = multishotTier × dmgVulnerability(custom) × headshotIncreaseDmg × headshotMult(non-crit) × viral × magnetic
              × stanceHitMult × heavyComboMult × slamMult × strength × additionalBuff × firstBulletMult × lastBulletMult
              × PlinxMag × DaikyuBroadhead × sniperCombo × electricityShardAbilityDmg × abilityCombo × overguardMult
innateSq    (V) = (1 − innateDR[layer])²                                                           (L10885)
shieldX3    (Ob) = 3 on the shield layer, else 1                                                    (L11576)
elemRes     (lb) = 1 − enemy.elemRes[type];  statusDR (Ra) = 1 − enemy.innateDR.status          (L10890-10894)
```

`statusCount` = `getCountDifferentStatuses` (§7). `dblMult` = 1 + `dbl_mult` (Mecha Empowered 1.5, L4890).

### 6.2 Tick helper `getTickStatusDmg(type, faction, baseDmg, armorMult)` (L9154-9204)

```
C        = 0.3 for Blast, else 0.5
elemBon  = custom.heatInherit if type == Heat and > 0, else Σ mods with element == type    (getStatValFromModElements, L8749-8762)
extra    = Toxin: add_toxin + unique + add_toxinS × strength ; Electricity: addElectricity ; Cold: addCold ; Heat: addHeat
raw      = C × baseDmg × (1 + elemBon + extra) × faction × mirage × dblMult
health   = typeMod(raw)                 typeMod = calcPercentAdd(x, typeOfFaction[faction][type])  (×1.5, ×0.5, or ×1)
shield   = typeMod(raw)                 (only if the enemy has shields)
overguard= Void ? raw × 1.5 : raw
armor    = typeMod(typeMod(raw)) × (1 − DR(armor × max(armorMult,0))) × (1 − innateDR.armor)
```
- `baseDmg` already contains `faction`, so DoTs get faction **twice**, matching the wiki (Faction_Damage_Bonus; wiki-status §3).
- `elemBon` uses only mods whose element **is** the proc type (Heat mods for Heat; only Gas mods for Gas; Toxin mods count even if combined into Corrosive). Matches wiki-status §3.
- Armor ticks apply the faction type modifier **twice** (Q12). `calcPercentAdd(b, m)` (L8182-8185) returns `b·m` if m > 1, `b − b·m` if 0 < m ≤ 1, else b; correct only because the table holds 1.5 and 0.5.
- `dblMult` is applied here and again inside `baseDmg` → squared for elemental DoTs (Q13).

### 6.3 Per-proc damage (status loop, L10883-11363)

For each status string `s` in the hit (`ca`), skipped entirely if immune to "all". Each proc stores `status[s].d` (crit path) and `.nc` (non-crit base path) per tick, and adds `curr_duration × d` to the hit's status sum `Bb` (`kc` for nc).

| Type | Per-tick value (health layer) | Ticks credited | Lines |
|---|---|---|---|
| Slash | `0.35 × kb × faction × V × Ob × (zb + unique.status_damage_slash) × hitMults(incl. Ma) × finisherMods(if enableFinisherModsWithSlash) × critMult × lb × Ra` — no type modifier, **no armor reduction**, no mirage/dblMult extra | `curr_duration` | L10904-10987 |
| Toxin | `getTickStatusDmg(Toxin)[layer] × (1 − innateDR) × Ob × (zb + status_damage_toxin) × hitMults(incl. Ma) × critMult × lb × Ra`; **skipped on the shield layer** | `curr_duration` | L10988-11058 |
| Heat | `getTickStatusDmg(Heat)[layer] × V × Ob × (zb + status_damage_heat + …) × hitMults(incl. Ma) × critMult × lb × Ra` | `curr_duration` | L11059-11144 |
| Electricity | `getTickStatusDmg(Electricity)[layer] × V × Ob × (zb + status_damage_electricity) × hitMults(incl. Ma) × critMult × lb × Ra` | `curr_duration` | L11145-11218 |
| Gas | `getTickStatusDmg(Gas)[layer] × V × Ob × (zb + status_damage_gas) × hitMults(**without Ma**) × critMult × lb × Ra` | **current Gas stacks on target** (or `curr_duration` if none), and only for the **first** Gas proc of the hit | L11219-11290 |
| Blast | `getTickStatusDmg(Blast)[layer]` (C = 0.3, Blast element mods) `× V × Ob × zb × hitMults(incl. Ma) × critMult × lb × Ra` | `curr_duration` = `parseInt(1.5 × SD)` (1 at SD 0) | L11291-11361 |
| Impact, Puncture, Cold, Corrosive, Magnetic, Radiation, Viral, Void, Tau, Microwave | no damage; debuffs only (§5) | — | — |

- `critMult` is the applying hit's full multiplier (tier, headshot, Cold, stealth). `hitMults` includes the Viral/Magnetic multiplier **at the time of the applying hit**; the wiki says Viral is read when each tick lands (wiki-status §4.13). Snapshot simplification.
- Cascadia Empowered (`dmgOnStatusEff` 750, L5174) via `calcDmgOnStatusEff(type, 750, layer, faction × viral × magnetic × elemRes, armorMult)` (L10240-10253, called at L10896-10903): for every elemental proc (base or combined, not physical) `750 × typeOfFaction[faction][type] × faction × viral × magnetic × elemRes`, armor layer `× (1 − DR) × (1 − innateDR.armor)`. No crit.
- Unique-enemy branches (Demolisher, Jugulus, Acolytes via `acolytesMetric` L10283-10294, Archon via `getArchonDR` L13265-13268, etc.) are **dead**: `showEnemy` overwrites every `unique` with `"notset"` (L7690), so `swithTypeUniqueEnemyDR` returns 1 (L13142-13191). The page states these enemies are treated as normal (index.txt L173).

### 6.4 Primary Debilitate damage (L11431-11462, L11612-11643, L11791-11822)
Only Heat, Toxin, Electricity extra procs deal damage (`lc` keys, L11399):
`ceil(getTickStatusDmg(type, faction, kb or fa)[layer] × (1 − innateDR) × critMult × Ob × hitMults × faction × (zb + typeStatusDmg))`, credited `× curr_duration` (L11986-11993). The extra `× faction` gives faction **three times** (Q14). `debilitateTickDmg` (L13207-13214) has no call site (dead code).

### 6.5 Toxic Lash (`toxicLashDmg`, L13215-13225; calls L11463-11497 etc.)
```
pct       = (melee ? 2 : 1) × option% / 100
direct    = ceil(hitDamage × mults × faction × pct)
tick      = ceil(hitDamage × faction × pct × 0.5 × (1 + ΣToxin element mods) × faction × mults × (zb + status_damage_toxin))
credited  = direct + tick × ToxicLash.curr_duration (7 at SD 0)        (L12001-12003)
```
`hitDamage` is the hit's per-layer damage (weapon mix, already type-modded and armor-reduced), so the extra hit inherits the weapon's type modifiers instead of Toxin's (Q15). Faction: once in `hitDamage`, then ×1 direct / ×2 tick → 2× and 3× total, matching Toxic_Lash.wiki ("triple-dipping"). Adds one Toxin stack per hit to the timeline.

### 6.6 Xata's Whisper (`getXataDmg`, L13226-13243)
`round(Σ attack modded damage × mults × faction × (OG ? 1.5 : 1) × armorTerm × xata%/100)`, `armorTerm = 1 − DR(armor × armorMult × (1 − innateDR.armor))` (innate DR inside the DR argument, Q16) or `1 − innateDR[layer]`. The multiplier passed is `Jb × headshotIncreaseDmg × HS × faction × dmgVulnerability` (L11763) although `Jb` already holds those terms (double-dip), and `21 × Jb × …` in the two non-crit branches (L11403, L11584) (Q17).

### 6.7 Nightwatch Napalm / Reconifex (L10409-10441, L10770-10819, L11965-11984)
- Napalm (`nNapalm` on "Rocket Explosion"): direct 150 and DoT base intended 75, but L10416 assigns `Nc = Xb` (150) — the 75 is discarded (Q18). Both × progenitor bonus (Tenet/Kuva/Coda) × faction (DoT faction²) × baseMods × (DoT: 1 + Heat mods) × mirage × min(6, seconds left) × viral × magnetic × zb; on the armor layer `getOgrisNapalmArmorDecrease(x, armorMult) = x × (1 − DR(armor × max(armorMult,0) × (1 − innateDR.armor)))` (L13244-13252, called at L10818-10819); on the other layers the value is used without armor reduction. Napalm strips armor like Heat (§5.3).
- Reconifex `reconifexHeat`: `status.heat.d += round(hitDmg × r / 2) × faction × heatRes × statusDR`, direct `+= hitDmg × r`, extra credit `Heat.curr_duration × heat.d` (L11965-11984).

### 6.8 Per-hit total (L11998-12011)
```
hitTotal = directHit (incl. Σ_procs curr_duration × tick, i.e. Bb)
         + Xata + ToxicLash(direct + tick × duration) + Debilitate + napalm direct + napalm DoT + Reconifex DoT + Cascadia
```
In the default (crit) branch `directHit = (layerDmg × Jb + glaive) × (1 − innateDR) × uniqueDR + Bb` (L11957-11960). In the two non-crit-base branches the status sum is inside the `(1 − innateDR)` bracket (L11575, L11756), so innate DR hits status damage a third time there.

The random-queue DPS is the median of 20 one-second buckets built by spreading each hit's `totalDmg` evenly by **hit index** (`getDmgPerSecond`, L12787-12805), not by hit time. All DoT damage therefore lands at its applying hit: no tick delay, no truncation at the 20 s boundary. The TTK simulator schedules real ticks instead (L13670-13711): `delay + k × duration/ticks` using the **unmodded** `duration` and `ticks`, Heat as one shared refreshing timer (L13690-13703), Gas limited to 10 live clouds (L13676-13681), and Blast skipped because `ticks == 0` (L13669) (Q19).

## 7. "Damage per status" mods (Condition Overload, Galvanized)

`getCountDifferentStatuses(attack)` (L12015-12020):
```
if "Manual number of statuses" option: return option value
n = number of keys in attack.damage (modded, after element combination)
if Viral not among them and the external-Viral option is on: n += 1
```
- It does **not** read the timeline. The count is the number of damage types the weapon deals, assumed always present on the target; actual proc probability, immunities (other than "all", L10520-10521) and procs of types the weapon does not deal (Hunter Munitions/Internal Bleeding Slash, stance procs, Encumber, Debilitate, Cryogenic Cold) are ignored. Damage types with no status (e.g. Finisher) would still count.
- Used for `base_per_status` (Galvanized Aptitude/Shot 0.8 or 1.2, Condition Overload; L100-172) in kb (§6.1) and in the per-type damage (L9226-9237, L9374-9396), and for `critFlatChancePerStatuses` (L10522, L8230). Not used for AoE attacks or slams (L10827; guide.txt L16).
- Additive vs multiplicative with base damage mods is per attack (`isCoMult`, e.g. Catabolyst "Partial Reload Impact" in samples/weapon-Catabolyst.json) or global (`optMultiplicativeBasePerStatus`, L12103-12105). Formulas in §6.1.
- `base_per_cold` (Secondary Shiver 4.5, L4556) is added whenever conditions are on, without checking Cold stacks.
- `flat_crit_with_status` (L10752-10760) is the only per-status feature that reads the live timeline (total stacks < 3).

## 8. Status chance > 100% and multishot (summary)
- SC > 100%: `floor(SC/100)` guaranteed procs + one with the rounded fractional chance, each type drawn independently (§3.3). Matches wiki-status §2.1.
- Multishot (non-beam): `floor(ms)` pellets + 1 with the rounded fractional chance per shot (L10050-10062); each pellet rolls crit and status independently and is its own hit in the timeline.
- Beams: one hit per tick; rolled multishot tier multiplies both damage (`Jb`, `Ya`) and SC (§3.3), so DoT damage scales with multishot twice; forced procs (Hunter Munitions, Internal Bleeding) are per tick, not per beam. Matches Multishot.wiki §Continuous Weapons.

## 9. Small helpers in scope

- `getTotalBaseDmg(i)` (L9792-9797): Σ of `currWeaponInitial.attacks[i].damage` (unmodded base, after progenitor/zaw additions).
- `hasToxinInDmg(attack)` (L9798-9800): attack has a `toxin` key; TTK uses it to send direct Toxin to health while shields are up (L12706-12707).
- `getLichDmgPart(lichElement, i)` (L12879-12884): `lichElement.val × Σ currInitialAttacks[i].damage`, the progenitor bonus added to a combined element containing it (L12947-12955).
- `addDmgToBase(flat)` (L13193-13206): scales every type by `(Σbase + flat) × damage_mult / Σbase` (zaw grips/links, L14585, L14613).
- `getStatusDamageValue` / `getStatusDamageSum` (L12197-12216): display only; `d` (or `nc` when the hit used non-crit base) × `curr_duration` × number of procs of that type, plus Toxic Lash ticks. Disagrees with the credited total for Gas (§6.3).
- `procMultishotArray` returns `{total_shots: n, mask: all 1}` for beams (L10053).

## Suspected inspiration quirks

- Q1 Type mask uses `ceil(share)` slots and draws from 0-99, under-weighting the last damage types by up to n−1 points (L12243, L10022).
- Q2 Fractional status chance, crit chance and multishot are rounded to whole percents before rolling (L10005, L10012, L10679, L10054).
- Q3 Unmatched mask index removes `1 + floor(SC/100)` entries (`splice(J, 1+m)`, L10032); only reachable when every type is immune.
- Q4 Hunter Munitions needs a crit and then rolls `0.3 × CC` (L10762-10765): effective chance CC²×0.3 instead of CC×0.3.
- Q5 A hit benefits from its own Viral/Magnetic/Corrosive/Cold/Puncture stacks and later pellets of the same shot benefit from earlier pellets (L10106-10112).
- Q6 Viral and Magnetic multipliers never reset to 1 after expiry (L10320-10321, L10779-10804).
- Q7 Heat strips 50% instantly while any Heat stack exists; no ramp, no ramp-back (L10815). Heat stacks expire individually; the wiki's refresh-all-stacks rule is ignored in the DPS (L11139-11143), so sustained Heat is undervalued.
- Q8 Enemy armor in (0, 200] is raised to 200×(1−d) (L8115); no game rule found for this.
- Q9 Cold adds 0.05/stack (+0.5 at 10) to the final multiplier, not 0.10+0.05(n−1) before the tier; also applied to AoE (L10821-10825).
- Q10 Puncture duration 6 s (wiki 10 s); Impact 1 s (wiki 6 s per stack); Blast max Infinity, so no 10-stack detonation (L5545-5571).
- Q11 Tier for a failed roll uses `CC − 99`, giving one tier too many when CC mod 100 is in (99, 100) (L9826).
- Q12 Armor-layer DoT ticks apply the faction type modifier twice (L9192, L9199).
- Q13 `dbl_mult` is squared for elemental DoTs (in kb L10843 and again L9157), once for Slash and direct.
- Q14 Debilitate procs multiply faction a third time (L11459).
- Q15 Toxic Lash damage inherits the weapon's damage-type modifiers instead of Toxin's (L11465-11495).
- Q16 Xata and Napalm put innate armor DR inside the armor DR argument (L13236, L13249); elsewhere it multiplies damage.
- Q17 Xata multiplier double-counts headshot/dmgVulnerability terms and uses a ×21 factor in non-crit branches (L11403, L11584, L11763).
- Q18 Napalm DoT base 75 is overwritten by 150 (L10415-10416).
- Q19 TTK ignores status duration mods (base `duration`/`ticks`) and never applies Blast status damage (L13669-13684).
- Q20 Status DoTs (except Toxin and Debilitate) use innate DR squared, `(1−innateDR)²` (L10885); Toxin uses it once.
- Q21 All non-Toxin DoTs are ×3 on the shield layer (`Ob`, L11576); no game source found.
- Q22 Gas credits `stacks on target × tick`, once per hit, instead of `duration × tick` per proc (L11283-11289); Gas nc path also multiplies faction again (`fa × oa`, L11227), as does Blast (L11299).
- Q23 Blast status damage scales with status duration (`parseInt(1.5×SD)` "ticks": 3× at +100% SD, 0 below −33%) and uses Blast element mods, which the Blast wiki page says do not apply (L9177, L11356-11360).
- Q24 Gas ignores the non-crit headshot multiplier `Ma` (L11230-11251); Slash ignores mirage and the second `dbl_mult` that the elemental DoTs get.
- Q25 Per-element status damage (Toxin shard etc.) is added to the product `zb`, so it is not scaled by `multiple`, frozen or `vuln_status_damage` terms (L11003, L11084).
- Q26 Encumber: at most one per pellet (not per shot) in `procStatusArray`; uses `round(random·12)`, so Impact and Viral get half weight (L10036, L12875).
- Q27 `maxCountProc` (boss cap of 4) limits debuff stacks and same-type procs per hit, but not DoT damage (every proc is credited).
- Q28 "Combine attack effects" merges statuses of different attacks by hit index, not by time (L12540-12545).
- Q29 `dmgMultAgainstFrozen` (Melee Careen) is always applied, frozen or not (L10466, L10501).
- Q30 Condition Overload count = number of weapon damage types, never the statuses really on the target (L12015-12020).
- Q31 Status duration is truncated to whole seconds for every status, including debuff windows (L9090-9093).
- Q32 Melee Duplicate uses the displayed attack SC, which for Tau weapons assumes max Tau stacks (L9459-9467, L10721).

## Open questions

- Why DoT damage on shields is ×3 (`Ob`, L11576): intended mechanic, or a leftover?
- Gas crediting by current stacks (L11283-11289): a deliberate model of the cloud's growth, or a bug?
- The ×21 in the non-crit Xata branches (L11403, L11584): unclear purpose.
- The armor ≤200 → 200 rule (L8115): unclear purpose.
- `acolytesMetric` (L10283-10294) formula `dps = ms·rate/(t/(CM−t)+1)·dmg·(shotgun?0.5:1)`, `t = 1 − 1/tier`, `moddedCM = ((1+0.5(CM−1))/CM)·(1+tier(CM−1))`: derivation unknown; dead code (unique reset to "notset").
- Semantics of `archon_vitality` (one extra Heat proc per hit with Heat) and `decr_armor_by_punc`, `addPunctureStatus`, `flat_crit_with_status`, `sagekPassive`: no mod with the last four was found in the mod table (L2-5435); they may be weapon `unique` stats.
- Whether `currWeapon.attacks[i].speed` passed to `setProcsITS` is the modded fire rate (Internal Bleeding threshold): the value is used as-is (L12537); the stat pipeline file should confirm.
- Shotgun per-pellet status chance: `getFinalStatusChance` uses `attackInitial.status_chance` per pellet hit; whether the data's shotgun SC is per pellet or per shot is a data-model question.
