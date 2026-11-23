Status: draft

# Inspiration optimizer ("WFOpt" / "Autobuild") — reverse engineering

Source: `research/external/inspiration/min.pretty.js` lines 15252-22038 (the IIFE `var WFOpt = (function () {...})()`),
read in full. All `Lnnnn` references are to that file. Function names are the minified names inside the IIFE; they are
local to WFOpt and are unrelated to same-named top-level functions elsewhere in the bundle. The changelog dates the
feature: "2026.10.30 An automatic builder for the best builds has been released", "2026.11.19 A 'lock parent element'
feature has been added to assembly optimization" (index.txt L211-213).

Related findings this file depends on: `inspiration-dps-aggregation.md` (what `queue_hp`, `TTK_Queue`, `TotalDmg_Queue`
contain), `inspiration-data-model.md` (mod table fields `prefer`, `onkill`, `uncomp`, `type`, `info`, slot ids).

---

## 0. One-paragraph summary

WFOpt is a heuristic, multi-phase local search that drives the **whole main calculator** (`reCalc()`, the random-queue
simulation) as a black-box scoring function. It does not have its own damage model. To make the black box usable for
search it (a) swaps the calculator's random number source for a deterministic, low-discrepancy "stratified" source with a
per-evaluation seed (common random numbers across candidates), (b) takes the median of 3 seeds per score, (c) memoises
scores by build signature, and (d) re-confirms every accepted change on an independent set of seeds. The search is:
enumerate element-mod configurations on a bare weapon (phase 3) → prefill warframe/shard/perk slots (3.5) → a portfolio
of up to ~18 starts (element config × top-3 weapon arcanes) each completed by a beam search over the free main slots
(phase 4) → repeated coordinate-ascent "sweeps" over every slot group plus weakest-slot replacement and pair swaps until a
fixed point (phase 6) → optional per-buff rebuilds (phase 7) → a "shake" that rebuilds the three weakest slots jointly →
an honest (true-random, many-run) choice between all builds found → an honest polish of the 2 weakest slots → a 30-run
honest final report. It ignores mod drain, capacity and polarity entirely, never generates rivens, optimises one attack
against one defence layer, and has several hard-coded exclusions and biases.

---

## 1. Entry points and phase pipeline

### 1.1 UI buttons → functions (Ya L21471-21719, listeners L21630-21718)

| Button (UI text) | Function | Purpose |
|---|---|---|
| "Build it" / "Abort" (`wfoptStart`) | `Nd` L20832-21021 (abort: `pd` L16638) | Full pipeline (§5) |
| "Clear log" | `B` L15331 | |
| Diagnostics box (class `wfoptExclBox dNone`, L21610 — presumably hidden by site CSS): "Estimate check" | `fc` L16674-16762 | Compare stochastic vs stratified estimate (§6.1) |
| "1: profile" | `Yb` L16846-17084 | Print weapon/enemy/pools/stat envelope (§6.2) |
| "2: sensitivity" | `Zc` L17099-17439 | Synthetic-probe stat elasticity map (§6.3) |
| "3: elements" | `vd` L17820-18068 | Phase 3 alone |
| "4: assembly" | `$c` L18126-18614 | Phase 4 alone |
| "5: polish" | `Fa` L19695-19863 | Phase 5 (not part of the full run) |
| "6: converge" | `Ac` L20140-20307 | Phase 6 alone |
| "7: buffs" | `hb` L20528-20831 | Phase 7 alone |

The button that opens the modal is injected into `.modsBlock` with text "Autobuild" (L21500-21506).

### 1.2 Public JS API (return object L21932-22037)

`open, close, setLogSink, defaultCtx, runValidate(fc), runProfile(Yb), runSensitivity(Zc), runElements(vd), runBeam($c),
runPolish(Fa), runConverge(Ac), runBuffPortfolio(hb), runFull(Nd), verifyFinal(yd), applyLastBuild, getLastBuild,
getElementConfigs, freshHit(Qa), readMetrics(Ga), applyCtx(ec), setSlot(U), setProbe(W), setIncPerk(ya), evalOnce(Ua),
evalAgg(la), _pref{of:nb, adj:Vc}, _onkill{is:od, stripped:Jd}, _recQueue(ma), _normElem(rd), dmgString(Ub), _excl{...},
score(fa), readCtx(Tb), poolForSlot(Db), statEnvelope(qd), enableStrat(sa), disableStrat(Ja), setStratSeed, takeSnapshot(ja),
applySnapshot(Z), stop(pd)`. `setLogSink` (L21937-21939) redirects the log (`m` L15314-15330 checks `Od`). This API looks
built for headless testing and is a usable oracle hook: e.g. `WFOpt.runFull(WFOpt.defaultCtx({metric:"ttk"}))`.

`defaultCtx` (L21940-21983) defaults: metric "dps", first attack, defence = `J()` (armor if the enemy has armor else
health), headshot false, **noFaction true**, noOnKill false, **maxCond true**, keepExisting false, searchQueue 10,
heavy false, comboVal 12, all buffs off at their default values; `frameSlots:true` expands to all four frame options.

### 1.3 Full run `Nd` (L20832-21021) — exact order

```
Aa = ja()                         // snapshot of the user's whole state (L15463-15503), incl. user queueLength
Rb = []                           // portfolio of builds found (Ia L18748-18787)
ec(ctx); vc(ctx); dc(ctx)         // apply ctx, strip excluded mods, clear disallowed frame slots
if ≥2 main slots filled: Ia("your build as it was", tag "orig")
PHASE 3   vd(ctx)                 // element configurations   (L20868)
PHASE 3.5 Ta(ctx)                 // if ctx.prefill===true, or prefill!==false and metric!="ttk"  (L20875-20891)
PHASE 4   $c(ctx)                 // only if ≥2 main slots are free/unlocked (L20892-20911); then cd() applies za
oa(ctx)                           // fast-estimate trust check; may switch to honest sampling (L20916)
PHASE 6   Ac(ctx)                 // converge, ≤6 sweeps (L20920)
PHASE 7   hb(ctx)                 // only if ≥1 external buff is allowed (L20923-20928)
SHAKE     mb(ctx, score, 0); mb(ctx, score, 1)   (L20929-20974)
Md(ctx)                           // honest choice among ≤7 portfolio builds (L20976)
Pb(ctx)                           // honest polish of the 2 weakest slots (L20978)
yd(ctx, L)                        // 30-run honest final check + "build passport" (L20981)
ad(fixedProgenitor)               // push build into the main UI and reCalc (L18659-18705)
log cache hit rate, total time, build list (L20985-21012)
```
On any exception / abort ("cancelled") it logs "Stopped: …" and `sb(true)` restores the snapshot `Aa` (L21014-21020,
`sb` L16642-16656). Single-phase runs also restore the original build at the end except phases 4/5/6/7, which apply their
result via `Fc` (L18706-18721). The progress bar is `E(percent, text)` (L15336-15343); `F()` (L15344-15348) is
`await setTimeout(0)` to keep the page responsive — the whole optimiser runs on the UI thread.

---

## 2. What the user can configure

Context object built by `Tb` (L16536-16584) from the modal form (`Ya` L21515-21600), applied by `ec` (L16585-16607).

| UI control (id) | ctx field | Range / default | Effect |
|---|---|---|---|
| Metric radio `wfoptMetric` | `metric` | "dps" (default), "ttk" ("Minimum TTK"), "hit_infight" ("Maximum hit damage against a protection layer") | Score function `fa` (§4.4). A 4th metric "hit_fresh" (first hit on a fresh enemy) is fully implemented (`fa` L16023, `la` L15921, `me` L21867, extra run counts) but **has no radio button**; reachable only via the API. |
| Attack `wfoptAttack` | `attack` | one of `currWeaponInitial.attacks[].name` | Only this attack is scored. |
| Defence layer `wfoptDefense` | `defense` | "auto" (= armor if armored else health, `J` L15410), health, armor, shield, overguard (only layers with value > 0, `y` L15403-15409) | Which per-layer result is read (`Hb` L15766-15797). For TTK the layer only toggles whether health is computed through armor (log text `V` L21245-21275). |
| Heavy attack `wfoptHeavy` + combo `wfoptComboVal` | `heavy`, `comboVal` | combo 1-13, default 12 | Sets globals `is_Heavy`, `comboСounterVal` (`ec` L16596-16598). Disabled if the attack has its own `isHeavy` (`Da` L21333-21346). |
| Search window `wfoptQueueLen` | `searchQueue` | 1-60 s, default 10; auto-set on open/attack change to `ma(attack) = clamp(round(100/attack.speed), 5, 20)` (L15422-15429, `qb` L21328-21332) | Sets `queueLength` (simulation length) during the search (`ec` L16599-16600). Warning if `speed*window > 250` shots or window < 5 s (`ka` L15430-15455). |
| Beam width `wfoptBeamW` | (read directly in `$c` L18156-18158) | 1-8, default 3 | Phase 4 beam width. |
| Exclude elements `wfoptElemExclude` | (read by `sd` L17792-17797) | free text, e.g. "gas, blast" | Phase 3 drops any configuration whose resulting attack damage contains an excluded type (`td` L17798-17806). |
| Use weak point damage `wfoptHeadshot` | `headshot` | off | Sets `is_headShot`; when off, headshot-only mods are excluded (`jd` L16241-16254, list `je` L21870-21875). |
| No faction bonus `wfoptNoFaction` | `noFaction` | **on** by default (L21575) | Excludes every mod with a `SMITE` stat (`jd` L16230-16235). |
| No on-kill bonuses `wfoptNoOnKill` | `noOnKill` | off | Excludes `onkill:true` mods whose whole effect is in `WITH_COND` (`jd` L16236-16240, `Jd` L16510-16516); for mixed on-kill mods `U` deletes `actions.WITH_COND` (L15561-15565). |
| Mod conditions are met `wfoptMaxCond` | `maxCond` | **on** by default (L21581-21583) | Sets `isMaxCond` (WITH_COND values at max stacks). The final check re-runs with it off to show the dependence (`yd` L19293-19315). |
| Keep installed mods `wfoptKeepExisting` | `keepExisting` | off | Locks every non-empty non-incarnon slot (`Dc` L16303-16322 → `qc`); search fills only empty slots. |
| Keep progenitor element `wfoptLockProg` (only shown for Kuva/Tenet/Coda weapons, `xb` L17739-17745) | `fixedProgenitor = hc()` → `{elem, val}` (L17759-17768) | off | This is the changelog's "lock parent element". Otherwise all 7 progenitor elements are enumerated (§5.1). Value comes from the main page `#pElVal` clamped to 25-60 %, default 60 % (`tb` L17746-17758). |
| Use incarnon perks `wfoptIncPerks` | `incPerks` | auto-checked iff the weapon has incarnon data (`X` L21317-21327) | Adds the incarnon-perk group to the sweeps (`ac` L19395-19398, `gb`). |
| Warframe mod (1 slot only) `wfoptFrameMods` | `frameMods` | off | Allows slot 13 only (14-16 are cleared, `dc` L16330-16332). |
| Warframe arcanes `wfoptFrameArcanes` | `frameArcanes` | off | Slots 11-12. |
| Aura `wfoptAura` | `aura` | off | Slot 17. |
| Archon shards `wfoptShards` | `shards` | off | Slots 23-27 (duplicates allowed, uniform-set trial first). |
| Ability strength `wfoptStrength` | `strength` | 1-2000 %, default = current `strengthMult` | Sets global `strengthMult` (`ec` L16588). Scales the buffs below. |
| Allowed external buffs `wfoptBuff_<g>` + value `wfoptBuffV_<g>` | `buffs[g]`, `buffVals[g]` | table `ib` L21731-21795: Toxic Lash (val 30), Xata's Whisper (50), Roar (100), Eclipse (100), Nourish (50), External viral (stacks 10, max 10), Absolute critical chance (50) | Each allowed buff is switched on during the whole search (`ec` L16601, `g` L15264-15277) and gets its own scenario in phase 7. |
| "Prohibited mods for build" list (`wfoptExcl*`) | — | persisted in `localStorage["wfoptExclNames"]` (`Sb` L16468-16488); defaults `ae` L21848-21856: Mecha Empowered, Proton Snap, Arcane Blade Charger, Arcane Primary Charger, Arcane Arachne | Excluded from every pool (`jd` L16205). Search by mod name, locale name or stat key (`Gd` L16411-16423, `kd` L16424-16442). |
| (API only) `allowOneShot` | | false; the checkbox id `wfoptAllowOneShot` does not exist in the HTML, `Sa` L21406-21409 | Allows "next shot" conditional mods for DPS/TTK. |
| (API only) `prefill` | | undefined | Forces phase 3.5 on/off. |
| (API only) `noStrat` | | set internally by `oa` | Use honest sampling instead of stratification. |

Not configurable at all: mod drain / capacity / polarity / forma / Orokin reactor, mod rank (always the table value, i.e.
max rank), number of main slots (always 8: `Ba.main` = 0-7), riven stats, multiple attacks or weighted attack mix,
enemy (taken from the main page's current enemy, level, Steel Path, Eximus, `Yb` L16926-16944), stance combo choice.

---

## 3. Slot model and candidate pools

### 3.1 Slot ids (`Ba` L21814-21823)
`main` 0-7, `exilus` 8, `stance` 9 (`Ad`=9 L21825), `warcane` 10, `archArcanes` 28-29 (archguns), `inc` 18-22 (incarnon
perks), `frame` 11-12 (warframe arcanes), 13-16 (warframe mods), 17 (aura), `shard` 23-27.

### 3.2 Slot eligibility `Nc(mod, slot)` (L16169-16202)
Let `catHit = mod.tags ∩ weapon.categories ≠ ∅`, `compHit = mod.tags ∩ weapon.compTags ≠ ∅`,
`banned = mod.uncomptag ∩ weapon.compTags ≠ ∅`.
```
slot 8      : type=="exilus" && (catHit||compHit) && !banned
slot 9      : type=="stance" && compHit
slot 10     : !archgun && type=="weapon_mist" && catHit && !banned
slot 11,12  : type=="frame_mist" && catHit
slot 13-16  : type=="frame_mod" && (catHit||compHit)
slot 17     : type=="aura_mod" && catHit
slot 23-27  : type=="shard" && (catHit||compHit)
slot 28     : archgun && type=="weapon_mist" && "primary" ∈ tags
slot 29     : archgun && type=="weapon_mist" && ("secondary" ∈ tags || catHit)
slot 18-22  : false (incarnon handled separately)
main 0-7    : false if mod.trigger=="Semi" && weapon.trigger!="Semi"; false if type defined and type!="exilus";
              else (catHit||compHit) && !banned      // exilus-type mods ARE allowed in main slots
```

### 3.3 Pool `Db(slot, ctx)` (L16443-16458)
All `modsProps` entries with `name`, `action`, **not** `action.unique` (only the two augment mods with UI blocks, Baruuk
and Excalibur, L4910/L4917), `Nc(mod,slot)`, and `jd(mod,ctx) == null`. Exclusions from `jd` (L16203-16271), in order:
1. name in the prohibited list → excluded (kind "name").
2. the mod's only meaningful stat is `status_duration` → always excluded (kind "dur"; logged by `N` L21196-21204 as
   "the theoretical DPS grows, but nobody will wait longer for the enemy to die"). "Meaningful" = not in `Sc`
   (L21876-21890), not in `Ic` (L21796-21813), not `na`, not numeric 0. Stats are flattened with `Mb` (L16459-16467):
   base `action` minus `WITH_COND`/`FOR_WEAPON`, overlaid by `WITH_COND` values.
3. `noFaction` and the mod has `SMITE` → excluded.
4. `noOnKill` and `onkill===true` and all non-ignored stats are inside `WITH_COND` → excluded.
5. `!headshot` and every stat is a weak-point stat (`je`: crit_chance_weakp, headshot_mult, mult_for_head, headCritChance)
   apart from ignorable ones → excluded.
6. `mod.info` matches `/next shot|next attack|first shot after/i` → class "next_shot"; `/on ability cast|on casting|after
   casting/i` → "on_ability" (`ke`, `le` L21861-21862). Allowed classes per metric `me` (L21863-21869): dps, ttk,
   hit_infight allow `on_ability`; hit_fresh allows both. So "next shot" mods are excluded for every UI metric unless
   `allowOneShot` (API only). Ability-cast mods are always allowed (and evaluated with conditions met if maxCond).

Additionally `vc` (L16272-16302) removes already-installed mods that fall under these exclusions (and the slot-10 arcane on
archguns) before the search, and `dc` (L16323-16344) clears disallowed warframe slots.

### 3.4 Candidate subsets used by the search
* `Zb(ctx, [0, 8])` (L17628-17667) — **element mods** from the main+exilus pools: mods whose flattened stats contain
  `element` with any value > 0. Deduplicated by a signature of all their stats; among equal signatures keep higher
  `prefer`, then fewer extra meaningful stats. Each entry `{name, els:[{el,val}], sig, extra}`.
* `zc(ctx)` (L18069-18088) — **non-element main-slot candidates**: main pool mods without `element` that have at least one
  meaningful non-zero stat. Note: includes exilus-type mods (eligible in main slots).
* `Wd(mod)` (L18089-18095) — a mod's "family" = first key present from the priority list `be` (L21923-21930: base,
  flat_change_dmg, base_per_status, multiple, multSharpshot, multishot, crit_chance, …, phys), else first meaningful key.

### 3.5 Incompatibility
* `Cc(a, b)` (L16138-16153): symmetric lookup built once from every mod's `uncomp` list.
* `Oc(a, b)` (L18096-18098): `a` contains `b` as a substring, or `b` contains `a`, or `Cc(a,b)`. The **substring rule** is a
  heuristic that catches "Primed X"/"X", "Amalgam Serration"/"Serration" without data, but would also flag any pair where
  one name is a substring of the other.
* `Fb(name, slot)` (L16154-16162): some other installed slot's mod name contains `name` or `Cc`-conflicts with it. Note:
  only "installed contains candidate", not the reverse, for the substring part (the `uncomp` part is symmetric).
* `Ec(name, exceptSlots)` (L18099-18115) and `Ob(names, cand)` (L18116-18119): beam-time check against the beam entry's
  names and against mods installed in main/exilus/arcane slots outside the free slots (`Td`).
* Element-config enumeration (`wb` L17668-17738) rejects pairs with equal names, substring names or `Cc`.

### 3.6 Riven
A riven exists only if the user filled in the riven editor: `showRiven` (L7273-7339) writes `modsProps[9999] = {name:
"Riven", tags:["primary","secondary","melee","primary-archgun"], action:{…user stats…}}`. Because `Db` iterates
`modsProps` up to `length` (sparse index 9999) it becomes an ordinary main-slot candidate with fixed stats, eligible for
every weapon class (it has no `type`, no `uncomp`). If the riven has an element stat it is treated as an element mod in
phase 3 (`Zb`), otherwise as a non-element candidate (`zc`). The optimiser never invents or tunes riven stats.

### 3.7 Arcanes, exilus, stance, incarnon, warframe slots, shards
* Weapon arcane (slot 10): ranked as an "anchor" before the beam (phase 4) and re-optimised in every sweep (group
  `jc()` L19387-19389, `fill: !archgun`). Archguns: slots 28 and 29 via `Xb()` (L16166-16168).
* Exilus (slot 8): only in the sweep groups (`ac` L19390-19415) and as an element candidate source in phase 3.
* Stance (slot 9): group only for melee and only if the attack is not a slam (`ra` L15456-15458: attack type "s"/"hs").
  `ba` (L15608-15653) tries every stance; **the combo is always set to the first combo of the stance** (L15588-15607 `qa`,
  log text L15642-15650 "the combo is set to the first one on the list - pick another by hand"). If the slot was empty the
  best stance is installed even if it scores below an empty slot.
* Incarnon perks (slots 18-22): `gb` (L19430-19474). For each incarnon slot, every perk option whose `type` list matches the
  attack (`dd` L19416-19429: "normal attack" matches non-incarnon attacks, anything containing "incarnon" matches attacks
  whose name contains "incarnon", or exact attack name; empty type = all) and whose `action` is non-empty is installed and
  scored; the raw best (no `prefer`, no confirmation) is kept, slot by slot in key order. Not locked by keepExisting
  (`Dc` skips entries with `prop_id`).
* Warframe arcanes 11-12, warframe mod 13, aura 17: ordinary `ab` groups with `fill:true`.
* Archon shards 23-27: `ab` group with `dup:true, uniform:true` — first tries "all five the same shard" for every shard,
  then per-slot coordinate ascent allowing duplicates (L19478-19530).

---

## 4. Evaluation: how a candidate is scored

### 4.1 One evaluation `Ua(ctx)` (L15801-15875)
```
key = [attack, defense, queueLength, seed rc] ++ [slot + (":"+name | "p"+prop_id) for every filled slot]
      ++ ["L"+progenitorElem+":"+val] ++ ["C"+stanceCombo] ++ ["S"+stanceSelectIndex]  joined by "|"
      (only when stratification is on and !ctx.noStrat and no "__probe" mod is installed; else no caching)
if key in cache Pc: cacheHits++ ; return Pc[key]
clear dpsQueue/TTK_Queue/MedianTotal_Queue/TotalDmg_Queue; if stratified: Pa()   // reset RNG state to the seed
isQueueRun = true; reCalc()                       // the full main calculator, all attacks and layers
metrics = Hb(attack, defense)                     // on exception: {dps:0, …, failed:true}, error counted in Hc/gd
if cacheable && !failed: if cache size ≥ 20000 → Pc = {} (full flush); Pc[key] = metrics
```
The cache is also flushed whenever a buff is toggled (`g` L15265-15266), the context is applied (`ec` L16586) or a
snapshot restored (`Z` L15505). The key does not contain buffs/strength/headshot/maxCond, which is safe only because of
those flushes (see quirks).

### 4.2 Metrics read from one simulation `Hb(attack, defense)` (L15766-15797)
* `dps` = `currWeapon.info[attack]["queue_hp" | "queue_"+defense]` — the median per-second bucket of the queue
  (`inspiration-dps-aggregation.md` §7.3).
* `ttk` = last value of `TTK_Queue[attack][defense]` (null = not killed in the window).
* `avgHit` = last value of `TotalDmg_Queue[attack][defense]` = `round(getAvgQueueVal(...).total)` (L12719-12724): mean per-hit
  direct + status damage.
* `medHit` = last of `MedianTotal_Queue[attack][defense]`.
* `freshHit` = `Qa(defense, attack)` (L15726-15754): sum of `totalDmg + getStatusDamageSum` of all `arrInfo` entries that share
  the earliest `timeM` — the first hit (all pellets) on a fresh enemy.
* `ammoWindow` = `ammoSchedule.cost × shots in window`; `ammoToKill` = `cost × #shots with time ≤ ttk + 1e-9` (binary search).
  `ammoSchedule` is only produced for finite-magazine, non-battery, non-incarnon attacks (L12491-12500).

### 4.3 Stratified ("fast estimate") random source — `sa` / `Ca` / `ta` / `Pa` / `Ja` (L15688-15725)
`sa(seed)` (L15708-15718) replaces two globals for the duration of the search:
* `isProc(chance, range)` (original L9819-9821: `range>0 && Math.random()*range < chance`) → `Ca` (L15688-15704):
  ```
  Ca(a, c):                       // a = integer percent (all call sites pass round(frac*100), range 100)
    if a <= 0: return false
    p   = a / (c + 1)             // NOTE: a/101, not a/100
    key = a + ":" + c             // one accumulator per (percent, range) pair — shared by ALL mechanics
    acc = Qc[key] if present else u0(key, seed)   // u0 = FNV-1a(key) seeded by 2654435761*seed, mixed, /2^32 → [0,1)
    acc += p
    if acc >= 1: Qc[key] = acc - 1; return true
    Qc[key] = acc; return false
  ```
  i.e. error diffusion (a "sigma-delta" / Bresenham sequence): over N calls with the same percent, almost exactly N·p
  succeed, with a seed-dependent phase.
* `Math.random` → `ta` (L15705-15707): golden-ratio additive recurrence `Pd = (Pd + 0.6180339887498949) mod 1`,
  initial `Pd = (0.5 + (seed + 1)·φ⁻¹) mod 1` (`Pa` L15719-15722). This drives status-type selection
  (`Math.floor(100*Math.random())` into the weighted status table, L10022), random-element picks, etc.
* `Pa()` resets all accumulators and `Pd` before every `Ua` (L15835), so the same build + seed is fully deterministic and
  all candidates evaluated under one seed see the same "random" stream → common random numbers.
* `Ja()` (L15723-15725) restores the real `isProc` and `Math.random`.

### 4.4 Aggregated estimate `la(ctx)` (L15920-15951) and honest sampling `Wb(ctx, n)` (L15880-15919)
```
la(ctx):
  if metric == "hit_fresh": return Wb(ctx, 9)          // always honest for the first hit (bimodal)
  if ctx.noStrat:           return Wb(ctx, 5)
  if not stratified:        return Ua(ctx)              // one real-random run
  for D in 0..2:  rc = seed + Bd + D; r_D = Ua(ctx)     // 3 stratified seeds ($d = 3, L21841)
  return per-field median of r_0..r_2 (ttk = null if > half null); failed only if all 3 failed

Wb(ctx, n): temporarily disable stratification; n independent real-random Ua runs; per-field median (`va` L16115-16121,
            median = mean of the two middle values for even n); ttk = null if more than half the runs did not kill.
```
`Bd` is 0 except during re-checks, where it is set to `eb()` = `97 + 29·k` with k cycling 1..4095 (L15876-15879), so a
re-check uses a disjoint set of 3 seeds.

### 4.5 Score `fa(ctx, metrics)` (L16019-16051)
```
if failed: return -1e15
dps         : return dps
hit_infight : return avgHit
hit_fresh   : return freshHit
ttk:
  EHP = currHealth + currShield + currOverguard                       // armor is not added
  if ttk is null (not killed in window):
      return dps>0 ? (EHP>0 ? -(queueLength + EHP/dps) : -2·queueLength) : -1e12
  if ttk <= 0 (dies on the first hit):
      hit = avgHit>0 ? avgHit : medHit>0 ? medHit : dps
      return hit>0 ? (EHP>0 ? 0.001 + ln(1 + hit/EHP) : 40.001) : 0     // positive = overkill margin
  else:
      e   = avgHit>0 ? avgHit : medHit>0 ? medHit : 0
      dt  = attack.speed>0 ? 1/attack.speed : 0
      a   = (e>0 && EHP>0 && dt>0) ? e/(e+EHP) · dt · 0.3 : 0
      r   = max(0, min(ttk/2, ttk − (round(1000·ttk) − 0.5)/1000))     // sub-millisecond residue, in [0, 0.001]
      return −ttk + (a>0 ? a·r/(r+a) : 0)                                // tie-break bonus < 1 ms
```
So TTK scores live on three disjoint bands: overkill (> 0), killed (−ttk), not killed (< −queueLength).
`Lc` (L16052-16060) maps a TTK score to a "bigger is better" magnitude for percentage reports: positive → `expm1(s−0.001)`
(= hit/EHP), negative → `1/(−s)`. `Kb(a, b)` (L16061-16073) = `100·(Lc(b) − Lc(a))/|Lc(a)|`, null if the TTK bands differ.

### 4.6 Comparison with mod `prefer` bias — `nb`, `Vc`, `$a`, `vb` (L16355-16388), `La`/`Ha` (L18788-18797)
* `nb(mod)` = `mod.prefer` clamped to [−25, 25]; only 5 mods have it (Magnum Force −1, Heavy Caliber −1, Secondary
  Dexterity −1, Primary Dexterity −1, Gilded Truth +3; see `inspiration-data-model.md`).
* `La()` = Σ `nb` over every installed regular mod (incarnon perks excluded).
* `Vc(score, pref)` = `Lc(score) · max(0.1, 1 + pref/100)`.
* `$a(a, prefA, b, prefB)`: for non-TTK metrics always `Vc(a,prefA) − Vc(b,prefB)`; for TTK the bias is applied only when
  the two prefs differ and both scores are in the same band (sign and "killed in window"), else raw `a − b`.
* `vb` sorts descending by `$a`. Every acceptance in the search uses `$a(...) > 1e-9` (or `1e-12`). When `prefer` changes a
  decision, `Ud`/`wc` (L16389-16404) log it ("The metric in the log is always the real one, with no bonus added").

### 4.7 Two-stage confirmation `kb(ctx, revert, apply)` (L16001-16018)
```
Bd = eb()                          // fresh seed block
revert(); old = fa(la()); prefOld = La()
apply();  new = fa(la()); prefNew = La()
Bd = 0
if $a(new, prefNew, old, prefOld) > 1e-12: accepted++ ; return {ok:true,  score: fa(la()) on the normal seeds}
else: rejected++ ; revert(); return {ok:false, score: fa(la())}
```
Used for every slot replacement in `ab`, `Ma`, `Bb`, the uniform shard set and the shake. Counts are reported by `ed`
(L21168-21178). Not used by `gb` (incarnon), `ba` (stance), `Jb` (element recount) or the beam itself.

### 4.8 Trust check of the fast estimate `oa(ctx)` (L15952-16000)
On the build after phase 4: `strat = fa(la())`, `honest = fa(Wb(ctx, 15))`, `dev = Kb(honest, strat)` (%). If
`|dev| > 80` (L15962) → `ctx.noStrat = true` for the rest of the run (every `la` becomes `Wb(ctx,5)`, i.e. 5 real-random
runs, and the cache is bypassed). The log explains why: exact frequencies remove the independence between rolls, which
breaks builds whose damage depends on the product of several conditions (L15988).

### 4.9 Sanity checks before any phase
* `mc` (L21042-21142): the chosen layer must have a positive pool, else abort. Logs the metric on the current build. Then
  installs a control probe `{base:9, crit_chance:2, crit_mult:2, status_chance:2}` in slot 0 (L21109); if the score does
  not change, the metric "DOES NOT REACT to mods" and the search aborts.
* `pc` (L21143-21167): for TTK, 3 evaluations; if ≥2 die on the first hit → "TTK IS SATURATED" (ranking falls back to the
  overkill band); if ≥2 are not killed → warns that ranking uses the EHP/DPS extrapolation.
* `V` (L21243-21316), `N` (L21179-21242): explanatory warnings (layer semantics, excluded mods, heavy/stance notes).

---

## 5. The search, phase by phase

### 5.1 Phase 3 — element configurations `vd` (L17820-18068)
1. `f = na(ctx)` (L15674-15682): clear main slots 0-7 unless keepExisting; `f` = list of empty main slots.
   Other slots (exilus, arcane, warframe…) keep whatever the user had.
2. `e = Zb(ctx, [0, 8])` element mods (deduped).
3. `l = min(3, |f|)`; `u = wb(e, l, 4000)` (L17668-17738): DFS over index-ordered subsets of size 1..l
   (stop at 4000 configurations). For each subset without incompatible pairs:
   * `G[el]` = summed element values; `O` = distinct elements in order of first appearance.
   * Element orders: if `|O| ≤ 2` → just `O`; else for each element P, the order "O without P, then P" (|O| orders).
     For 3 base elements this gives exactly the three possible outcomes (which element stays uncombined); for 4 it covers
     the three pairings.
   * Mods are re-ordered to follow the element order (each mod placed at its first element in the order); configurations
     that need all mods placed are kept; dedup key `order.join(">") + "|" + modNames`.
4. Progenitor: if the weapon is Kuva/Tenet/Coda (`xb`) and not locked, variants `pe` = toxin, cold, heat, electricity,
   magnetic, radiation, impact (L21921) at `tb()` (25-60 %); else one variant (current).
5. `sa(0)`; for each progenitor variant × configuration: clear `f`, install the config's mods in slot order into `f[0..]`,
   `score = fa(la())` (3 stratified seeds), drop it if the resulting damage contains an excluded element. Baseline per
   variant = score with no element mods.
6. Sort by `vb`; log the top 15, the best per number of slots, and the best per distinct "damage mix" string (`Ub`
   L17807-17815: attack damage types and rounded values).
7. Output: `Bc` = top 8 configurations (starts for phase 4) (L18039); `Kc` = up to 40 configurations with unique
   (progenitor, damage mix) (L18040-18054) — the shortlist re-tried in every phase 6 sweep (`Jb`).

Important: the elements are chosen on an otherwise **empty** weapon (no crit/status/multishot mods), only fixed by the
recount in phase 6.

### 5.2 Phase 3.5 — prefill non-weapon slots `Ta` (L20436-20527)
Skipped for TTK by default (log L20886: early filling pushes TTK into the one-shot band). Steps: install `Bc[0]`
element mods into the unlocked main slots; rank `zc` candidates in the first empty slot; greedily add up to 3 of the top
12 to empty slots (each accepted only if it improves) — a temporary "skeleton"; then run every `ac` group except stance,
weapon arcane and exilus (i.e. incarnon perks, warframe arcanes, warframe mod, aura, shards) with `gb`/`ab`; finally clear
the skeleton main slots again. Effect: warframe-side slots are chosen against a plausible weapon build before the beam.

### 5.3 Phase 4 — portfolio of starts + beam search `$c` (L18126-18614)
Requires ≥ 2 free main slots (else skipped; with 1 slot the sweeps and honest polish pick it).
1. Beam width `c` = `wfoptBeamW` clamped 1-8 (default 3) (L18156-18158).
2. Starts `r` = `Bc` (element configs, each with its progenitor unless locked); if phase 3 did not run, one empty start.
3. Candidates `C = zc(ctx)`; with keepExisting, candidates and element starts already installed in locked slots are dropped.
4. **Weapon arcane anchor ("Phase 2.5")** (L18245-18304): with start 0 installed, score every slot-10 arcane plus "no
   arcane"; keep the top 3 (drop a trailing "no arcane" if more than one) as anchors `D` (L18292-18295). Archguns instead run one `ab`
   pass on slots 28-29. If the user locked an arcane (keepExisting), it is the only anchor.
5. Portfolio: first `ceil(16/|D|)` element starts × `|D|` anchors (L18306-18321) → up to 18 starts (6 × 3).
6. For each start (L18334-18455):
   ```
   install progenitor, arcane anchor, element mods into f[0..k-1]; O = score
   P = [{names: elementNames, score: O}]          // the beam
   R = |f| - k                                     // slots left
   for level ha in 0..R-1:
       pool = (ha == 0) ? C : T                    // T = top 30 candidates of level 0 (per start)
       Y = []
       for each beam entry × each pool mod: skip if Ob() incompatible; skip if sorted name set already seen;
           install entry+mod into f in order; score; push
       if Y empty: break
       sort Y by vb
       if ha == 0:
           T = top 30 single-mod additions
           family diversity: best addition per family Wd(mod), families in score order, take up to max(c,4),
           top up with the best remaining additions until max(c,4) entries → new beam (wider first level)
       else:
           P = Y[0..c-1]
   record P[0] (names, score, start score O, damage mix)
   ```
   No backtracking inside a start; the element mods stay in the first slots (fixed element order).
7. Re-rank (L18456-18490): the top 6 starts are re-evaluated on 2 further independent seed blocks (`Bd = eb()`),
   `est = median(original, re-eval 1, re-eval 2)`, re-sorted; the log says when "the first one had just drawn a lucky
   sample".
8. Report: best build (slots, arcane, progenitor, damage mix), **leave-one-out contribution of each mod** (L18544-18585, threshold L18577:
   score without that mod, phrase "without it: lower by X%"; mods under 2 % are flagged "Weak slots"), and the next 5
   starts with their builds. `za` (L18598-18608) stores the result.

Cost per start ≈ |C| + (R−1)·c·30 evaluations (each = 3 stratified simulations unless cached).

### 5.4 Fast-estimate trust check — `oa`, see §4.8.

### 5.5 Phase 6 — converge to a fixed point `Ac` (L20140-20307) → `kc(ctx, 6)` (L20058-20139)
Up to 6 sweeps. Candidate list `u` = `zc` ∪ element mods from `Zb` (by name). Each sweep, in order:
1. **Slot groups** `ac(ctx)` (L19390-19415), each handled by:
   * weapon arcane (slot 10, or 28-29 for archguns) → `ab`
   * exilus (8) → `ab`
   * stance + combo (9; melee, non-slam) → `ba`
   * incarnon perks (if `incPerks`) → `gb`
   * warframe arcanes (11-12), warframe mod (13), aura (17), shards (23-27, dup+uniform) → `ab` (if allowed)

   `ab(ctx, group, score)` (L19475-19588), per non-locked slot of the group:
   ```
   cur = installed mod; best = current (score, pref)
   if cur: try empty slot; take it if $a(...) > 1e-9
   elif group.fill: best = -∞            // will take the best candidate, but kb() below still rejects a worse one
   for each pool mod ≠ cur that is compatible (or group.dup): install, score; keep $a-best (and raw best for logging)
   install best; if it changed: kb(revert→cur, apply→best) two-stage confirmation
   ```
2. **Element recount** `Jb` (L19864-19925): slots G = main slots that are empty or hold an element mod (sorted). For each
   shortlisted config in `Kc` (≤ 40, from phase 3, with its progenitor unless locked) that fits in |G|: clear G, install
   the config into G in order, score; keep the `$a`-best. **No confirmation, and it ignores keepExisting locks.**
3. **Free main slots**: any still-empty unlocked main slot is filled with `ab({fill:true})` (L20102-20115).
4. **Weakest-slot replacement** `Ma(ctx, main, u, score, 4)` (L19589-19694), up to 4 iterations:
   ```
   weakest = unlocked filled slot whose removal gives the highest ($a) score
   remove it; option "leave empty" if that beats the current score
   try every candidate in u (≠ removed mod, not tabu slot|name, compatible); keep the $a-best
   if a replacement won: kb() confirm; rejected → tabu[slot|name]; if nothing won: restore and stop
   ```
5. **Pair swaps** `Bb(ctx, score, u, 2)` (L19926-20057), up to 2 iterations: the 3 weakest slots (by removal score); the
   top 12 replacements for the weakest slot (excluding mods already installed); for each pair of the 3 slots and each
   unordered pair of the 12 candidates (≤ 3 × 66 evaluations) install (a→first slot, b→second slot); best improving pair is
   confirmed with `kb()`; rejected pairs become tabu.
6. Convergence (L20133-20136): stop if no group changed, or if Σprefer is unchanged and the sweep gained < 0.3 %
   (< 2 % for hit_fresh).

Report: per-sweep score/gain/changed groups, final build, final damage mix, progenitor, leave-one-out contributions of the
main slots, calculation failures (`Hc`, first error `gd`), confirmation stats, `prefer` interventions. The result is
added to the portfolio `Rb` with tag "cur".

### 5.6 Phase 7 — external buff portfolio `hb` (L20528-20831)
Only when at least one buff is allowed. Scenarios: "no buffs", each allowed buff alone, "all allowed" (if > 1)
(L20594-20596). For each scenario: restore the post-phase-6 snapshot, switch on that scenario's buffs, score the original
build (`sBase`), run 2 quiet sweeps `kc(ctx, 2)` (`zd` silences the log), score again (`sOpt`); keep the rebuilt panel only if
it beats `sBase`. Xata's Whisper share is reported per scenario (`rb` L15755-15765 = 100·Σxata/Σtotal). Tables: "What a
buff is worth on one and the same build" and "Is rebuilding for the scenario worth anything". Final pick: under the
**last** scenario's buffs (= all allowed, or the single allowed buff), evaluate the original and every rebuilt panel; keep
the best; its buffs stay switched on and the main page's checkboxes are synced (`ob` L16614-16637). Every evaluated
panel is added to the portfolio.

### 5.7 Shake `mb(ctx, score, variant)` (L20308-20435), run for variant 0 and 1
Ranks the unlocked filled main slots by removal score; variant v takes the slots ranked 3v..3v+2 (the 3 weakest, then
the next 3). Removes all three, ranks every candidate (non-element + element) in the first of those slots with a single
stratified evaluation (`Ua`, not `la`), keeps the top 12, then a beam of width 3 over the 3 slots (no duplicate names,
dedup by sorted set). The best triple is installed, **added to the portfolio regardless**, and confirmed with `kb()`;
otherwise the previous build is restored. Purpose (log L20345): "a pair or a triple that only works together could
never be found" by one-at-a-time measurements.

### 5.8 Honest choice `Md(ctx)` (L18880-18980)
Portfolio `Gc(7)` (L18798-18818): the user's original build (if recorded) + up to 6 others, "cur" first, rest by score.
Needs ≥ 2. Stratification off; window = the user's own `queueLength` from the snapshot (default 20 if missing, L18884), not
the search window. 15 rounds (27 for hit_fresh, L18895); in each round every build is evaluated once (`bd` L18819-18832: score,
primary = −ttk for TTK if killed, else score; ammo = ammoToKill (TTK) / ammoWindow (DPS)). `xd` (L18839-18879): per-build
medians; sort by primary then score (both via `$a`); **ammo tie-break**: among builds with the same Σprefer whose primary
is ≥ 99 % of the winner's (DPS) or equal / both one-shot (TTK), the one with the lowest median ammo is promoted to first.
The winner is installed; the log compares it with the original build.

### 5.9 Honest polish `Pb(ctx)` (L19032-19152) → `Xd` (L18981-19031)
Measures each installed main/arcane slot's removal score honestly (median of 3 real-random single runs at the search
window), warns about locked mods that hurt the score, takes the 2 weakest unlocked slots. For each (at the user window):
rank every compatible pool mod for that slot with one evaluation each (stratified seed 0, `sa(0)` at L18984), then compare
{current, empty, top 3 alternatives} with 9 honest runs each (17 for hit_fresh) via `xd`; the winner stays. "There is no
threshold here on purpose: the fast estimate is what put a mod in, so it does not get to defend it." (L19045-19053).

### 5.10 Final check `yd(ctx, searchScore)` (L19182-19386)
30 honest runs at the user window: median DPS (on the ranked layer) with coefficient of variation (`Cb` L16127-16132),
median TTK and the number of runs that did not kill, median ammo, avg/median per hit, first-hit damage, difference to the
search metric. If maxCond: 9 honest runs with conditions off ("How much of this rests on conditions being met").
"Build passport": the same runs read out for every other defence layer and every other attack of the weapon. First-hit is
reported but deliberately never optimised for in the UI (log L19327: it "spreads about 80-fold").

---

## 6. Diagnostic-only phases (not in the full run)

### 6.1 Estimate check `fc` (L16674-16762)
12 real-random evaluations (mean, CV, ms each); 6 stratified evaluations with seeds 0.137·k (bias vs the real mean,
warning if > 5 %); determinism check (two evaluations with seed 0 must match); cost estimate per 1000 evaluations.

### 6.2 Profile `Yb` (L16846-17084)
Logs attack base stats, innate damage, multishot/magazine/reload/charge/burst, enemy (faction, level, SP, Eximus, layers,
status immunities, `maxProcStacks` warning, external-viral note), per-slot pool sizes (and how many are elemental),
incarnon slot/variant counts, the "envelope" `qd` (L16763-16783: per stat key, best single mod, sum of top 3, count, best
mod) and mods with unmodelled penalties (`Vd` L16524-16535: accuracy/recoil/spread/zoom/d_sound).

### 6.3 Sensitivity map `Zc` (L17099-17439)
Uses a synthetic `__probe` mod (`W` L15568-15571) in a free slot. For each stat key (ordered by a priority list, element /
SMITE / phys skipped) a value ladder `Yc` (L17085-17098): `[0.5·best, best, Σtop2, Σtop3, Σtop4, Σtop_D]` where D = free main
slots (min 3). Logs score, gain %, elasticity (gain/value). Stats marked inapplicable by `Xc` (L16803-16841: no-damage
stats, no charge time, melee-only stats on non-melee, weak-point stats with headshot off, base_per_status on AoE/slam,
add_slash_on_impact without impact, magazine stats without a finite magazine) are probed once and only reported if they
still move the score > 0.5 % ("the rule is out of date"). If the probe gives 0, the cleanest real mod with that stat
(`Ld` L16784-16802) is tried ("the effect is gated by mod name"). Element test `Ib` (L17440-17505: each base element at its
best mod value, each combined element as two mods), faction test `xc` (L17506-17522), and a second pass `yc`
(L17523-17627) on a skeleton (best element + best mod of the top 3 families) that flags stats that "REVIVED" or "died".

### 6.4 Phase 5 polish `Fa` (L19695-19863)
One pass of the slot groups, then `Ma(..., 8)` (8 weakest-slot iterations), then leave-one-out contributions. Superseded
by phase 6 in the full run.

---

## 7. Constants (L21720-21931)

| Name | Value | Meaning |
|---|---|---|
| `$d` | 3 | stratified seeds per estimate |
| `ee` | 5 | honest runs per estimate after the trust check fails |
| `ie` | 9 | honest runs per estimate for hit_fresh |
| `he` | 20000 | cache capacity (full flush when reached) |
| `Yd` | 0.6180339887498949 | golden-ratio step for the `Math.random` replacement |
| oa tolerance | 80 % | L15962 |
| `wb` limit | 4000 configs, ≤ 3 element mods | L17879-17880 |
| `Bc` / `Kc` | top 8 / ≤ 40 unique damage mixes | L18039-18054 |
| arcane anchors | top 3 | L18293 |
| portfolio | ceil(16/anchors) element starts × anchors | L18306 |
| beam | width 1-8 (default 3), first level max(width,4) with family diversity, candidates after level 0 = top 30 | L18156-18158, L18395-18433 |
| re-rank | top 6 starts, +2 seed blocks | L18457-18466 |
| phase 6 | ≤ 6 sweeps; Ma 4 iterations; Bb 2 iterations (3 slots × 12 candidates); stop at < 0.3 % (2 % hit_fresh) | L20064, L20116-20136, L20194 |
| phase 7 | 2 sweeps per scenario | L20618 |
| shake | 2 variants × 3 slots, top 12, beam 3 | L20947-20948, L20362-20395 |
| honest choice | ≤ 7 builds × 15 rounds (27 hit_fresh) | L18882, L18895 |
| honest polish | 2 slots, 3 alternatives, 9 runs (17 hit_fresh) | L19103, L19007 |
| final check | 30 runs; 9 runs for "conditions off" | L19224, L19304 |
| weak slot | < 2 % leave-one-out drop | L18579-18585 |

---

## 8. What it does not model or search (gaps vs. a "real" build optimiser)

* **No drain / capacity / polarity / forma / Orokin reactor / mod rank.** Neither the word drain nor polarity appears in
  the optimiser (grep over L15252-22038). Any 8 main mods + exilus + arcane are accepted; all mods are at the table's
  (max-rank) values. A found build may be unequippable on the user's weapon.
* No riven generation (only a user-entered riven as a fixed candidate, §3.6).
* No set-bonus awareness (mods' `set` field is copied by `U` L15560 but the search treats them independently).
* One attack and one layer per run; the "passport" only reports other attacks/layers. No weighted multi-attack objective,
  no combined primary+heavy rotation.
* No "burst" metric distinct from DPS: DPS is the simulated window (default search window 5-20 s), which includes reloads;
  a short window approximates burst only incidentally.
* Stance combo never optimised (first combo); no per-attack combo counter search beyond the user's fixed value.
* Warframe mods limited to one slot (13); warframe abilities only as the 7 fixed buffs; no companion/other-weapon effects
  (arcanes depending on another weapon are excluded by default list `ae`).
* "Next shot" conditional mods excluded for DPS/TTK; status-duration-only mods always excluded.
* No explicit user "must include / must exclude element" besides the exclude-elements text field; no "keep this slot"
  per slot (only all-or-nothing keepExisting).

---

## 9. Known weaknesses (our assessment, with evidence)

1. **Scoring is the full stochastic simulator.** Every candidate costs 3 complete `reCalc()` runs over all attacks and
   layers (createDmgQueue loops every attack, `inspiration-dps-aggregation.md` §7.5) plus DOM work inside reCalc
   (e.g. `J.append(createDamageInfoHTML(...))` L12725-12727 runs on every evaluation). The search is therefore limited to
   a few thousand evaluations and needs aggressive pruning.
2. **The fast estimate is biased** and the author knows it ("The search metric is noisy and biased upward", L18910):
   correlated rolls from the shared per-percent accumulator, the a/101 bias (§10 Q1), and the winner's curse from picking
   the max of many noisy scores. The honest-choice/polish stages only re-check ≤ 7 builds and 2 slots.
3. **80 % trust tolerance** (L15962) is so wide that a 50 % error in the fast estimate does not trigger honest mode.
4. **Elements chosen on a bare weapon.** Phase 3 ranks element configs with no crit/status/multishot mods; element value
   depends heavily on status chance and on viral/heat/corrosive interactions. Mitigated only by re-trying the ≤ 40
   shortlisted configs in each sweep (`Jb`), which are still phase-3 rankings.
5. **Element order is only searched in phase 3** ("move one element last"); later swaps (`Ma`, `Bb`, `ab` on free slots)
   insert element mods at arbitrary slot indices, implicitly changing combination order without considering alternatives.
6. **Beam pruning by marginal value**: after level 0 only the top 30 single-mod additions on the element skeleton are
   considered. Synergy mods that are weak alone (crit damage without crit chance, status damage without status) can be
   pruned; partially compensated by the family-diverse first level, `Bb` pair swaps and the shake.
7. **Greedy per-slot choices** for incarnon perks (no prefer, no confirmation, slot by slot), stance (forced, first
   combo), warframe slots (coordinate ascent).
8. **Order dependence**: groups are processed in a fixed order each sweep; the convergence loop reduces but does not
   remove it (max 6 sweeps, stop at < 0.3 %).
9. **Median of 3 seeds**, re-check on 3 more: small samples for heavy-tailed outcomes (crit tiers, status stacks).
10. **TTK metric** is quantised by shot times and needs a sub-millisecond tie-breaker hack (§4.5); saturated cases (one-shot
    or never-kill) collapse to proxies (overkill ratio / EHP÷DPS extrapolation).
11. **Single-threaded UI-thread JavaScript**, yielding with `setTimeout(0)` between evaluations: slow, and the page is busy.
12. **Exclusions by regex on tooltip text** (`ke`/`le`) and by hard-coded name list — brittle.
13. **Phase 7 picks under "the last scenario"** (all allowed buffs) only; a user who wants a build for one buff must
    allow only that buff.

---

## 10. Suspected inspiration quirks

* **Q1 — Stratified proc probability is a/(c+1), not a/c** (`Ca` L15690: `f = a / (c + 1)`; original `isProc` L9819-9821
  gives `a/c`). Every fractional roll during the search (crit tier, multishot, status count, Hunter Munitions, etc., all
  called with `range = 100`) succeeds at a/101: a relative −0.99 % bias per roll; a 99 % roll fails 2 % of the time.
  The honest stages use the real `isProc`, so search and honest numbers differ systematically. Likely a bug.
* **Q2 — One accumulator per integer percent, shared across mechanics** (`Ca` key `a + ":" + c`, L15691). A 30 % crit
  remainder and a 30 % status remainder interleave on one error-diffusion sequence, so joint events are deterministic
  patterns rather than independent. Author-acknowledged (L15988) and guarded by the 80 % check.
* **Q3 — `Math.random` globally replaced** by a golden-ratio sequence while searching (`sa` L15716): any other code path that
  calls `Math.random` during reCalc is also de-randomised (intentional, but global).
* **Q4 — `Jb` element recount ignores keepExisting locks** (L19874-19885 builds the slot list from empty slots and all
  element-mod slots without checking `qc`), so a locked element mod can be replaced. Suspected bug.
* **Q5 — `fill` groups are not really forced**: `ab` sets the baseline to −∞ for an empty `fill` slot (L19539) but the
  change still goes through `kb()`, which reverts if the new mod is not better than empty. Only the stance (`ba`) is
  truly forced. Harmless, but the log wording suggests otherwise.
* **Q6 — Substring incompatibility** (`Oc` L18096-18098, `Fb` L16158): any mod whose name contains another mod's name is
  treated as incompatible with it, even when the game allows both.
* **Q7 — TTK tie-break hack** (`fa` L16040-16049): adds up to 1 ms of bonus proportional to `hit/(hit+EHP)·shotInterval·0.3`
  and the sub-ms residue of ttk; it can reorder builds whose TTK differs by < 1 ms. EHP excludes armor.
* **Q8 — `prefer` multiplies the metric** (Gilded Truth +3 %, four mods −1 %): a deliberate bias toward/against specific
  mods, applied in every comparison (§4.6). For TTK the bias is applied to `1/ttk`.
* **Q9 — "Absolute status duration" mods are always excluded** (`jd` L16206-16228) even when the metric is DPS over a
  window where longer DoTs would matter.
* **Q10 — Honest choice uses the user's main-page window** (`Aa.queueLength`, default 20 s, L18884, also L19038, L19199) while the search used
  the search window (default 5-20 s by attack speed). The two can rank builds differently (logged in `yd` L19286).
* **Q11 — Shake ranks candidates with one stratified seed** (`fa(a, Ua(a))`, L20361) instead of the 3-seed median used
  elsewhere.
* **Q12 — Cache key omits buffs/strength/headshot/maxCond/customStats** (`Ua` L15803-15827); correctness relies on the
  cache being flushed whenever those change (`g`, `ec`, `Z`). `yd` toggles `isMaxCond` with stratification off, so it is
  uncached — fine — but any future code path that changes a global without a flush would read stale scores.
* **Q13 — Phase 3 runs with the user's existing non-main slots** (exilus, arcane, warframe slots are not cleared by `na`),
  so the element ranking depends on whatever was installed before the run.
* **Q14 — hit_fresh metric implemented but unreachable from the UI**; `allowOneShot` likewise (no checkbox, L21406-21409).

---

## 11. Open questions

* **OQ1** — Does `reCalc()` in queue mode skip any DOM work? `createDmgQueue` appends result HTML on every call (L12725);
  measuring the real per-evaluation cost would need running the bundle (see `oracle-feasibility.md`).
* **OQ2** — `.dNone` is not defined in the bundle; I assume the "Diagnostics by phase" panel is hidden by site CSS.
* **OQ3** — `Ca`'s hash uses `2654435761 * seed` as a float before `^` coerces to int32 (L15696); for non-integer seeds
  (`sa(0.137*f)` in `fc`) the exact start offsets depend on JS float→int32 truncation. Only relevant for bit-exact parity.
* **OQ4** — Whether a/(c+1) (Q1) is intentional (e.g. to mimic `Math.random()*d < b` with `Math.random` ∈ [0,1)) cannot be
  decided from the code; the honest path's probability is exactly b/d for integer b, so the two disagree either way.
* **OQ5** — How element ordering interacts with innate weapon elements and the progenitor element in the main calculator
  (combination order rules) is out of scope here; see the stat-pipeline/element findings.

---

## 12. OUR PROPOSAL — a multi-threaded Rust optimiser that matches and beats WFOpt

This section is our design proposal, **not** the inspiration's behaviour.

### 12.1 Principles
1. **Separate "what is a build" from "how good is it".** A build = (main-slot multiset, element order, exilus, weapon
   arcane(s), progenitor element+value, incarnon perk choices, stance+combo, warframe arcanes/mod/aura, shards, riven).
   Canonicalise it (sorted non-element mods + ordered element mods + the rest) so equal builds hash equal.
2. **Deterministic expected-value scorer as the search objective.** Our engine should compute expected damage per hit /
   per second analytically (expected crit tier from crit chance, expected multishot, expected status procs per type and
   their stacking over a window), which is deterministic, noise-free and roughly 10³-10⁴× cheaper than a simulation. This
   removes WFOpt's biggest weaknesses (noise, winner's curse, the a/101 bias, correlated stratified rolls).
3. **Simulation only for validation and for metrics that need it** (TTK with layer order, heavy tails, ammo/reload
   timing, condition uptimes). When simulating, use **common random numbers with independent streams**: a counter-based
   RNG (e.g. ChaCha/Philox keyed by `(run_seed, shot_index, pellet, mechanic_id)`) so every candidate sees the same
   random draws per mechanic, draws are independent across mechanics, and results are bit-reproducible across threads.
4. **Stat aggregation is additive per bucket** (base damage %, multishot %, crit chance %, …). Precompute each mod's stat
   vector once; a build's buckets are sums, so adding/removing a mod is an O(#stats) delta, and the score function is
   a cheap closed form over bucket totals. This makes millions of evaluations per second per core feasible.

### 12.2 Search strategy (anytime, parallel)
1. **Pool reduction.** Eligibility exactly as `Nc` + data-driven incompatibility (`uncomp` lists; no substring rule unless
   the data lacks a pair). Drop mods whose every stat is irrelevant to the chosen metric for this attack (WFOpt's `Sc`/`Ic`
   idea, but derived from the engine). Remove **dominated** mods: if mod A ≥ mod B in every relevant stat bucket (and same
   or lower drain when capacity is enforced), B can never be strictly better.
2. **Outer enumeration (embarrassingly parallel with rayon):** element configuration (all orders that produce distinct
   combined damage, including interaction with innate and progenitor elements) × progenitor element (if not locked) ×
   weapon arcane × exilus choice (or treat exilus as a 9th slot with its own pool). Each task is independent.
3. **Inner exact search for the remaining main slots:** branch-and-bound over the reduced pool on the analytic score.
   Upper bound: the score is monotone non-decreasing in each bucket for nearly all metrics, so a bound is obtained by
   adding, for each bucket independently, the best achievable sum from the remaining k slots (top-k values per bucket).
   With ~30-60 candidates and k ≤ 8 this prunes most of the C(n,k) space; fall back to a large beam (width 50-200,
   deterministic) when the bound is weak (e.g. non-monotone conditional effects).
4. **Local search polish** from the top N results of every task: 1-swaps, 2-swaps (WFOpt's `Ma`/`Bb`), element reorder,
   arcane/exilus/perk/stance coordinate ascent, all with exact deterministic deltas; optional simulated annealing / random
   restarts with fixed seeds for reproducibility.
5. **Joint handling of the non-weapon groups** (warframe arcanes, aura, shards, incarnon perks): small finite sets →
   enumerate jointly where the product is small (perks: ≤ 5 slots × few options), otherwise coordinate ascent inside the
   local search. Shards: multiset enumeration (5 slots from ~18 types is C(22,5)=26k multisets — cheap analytically).
6. **Final validation**: re-score the top K (e.g. 20) distinct builds with the stochastic simulator using CRN and many runs
   (e.g. 200-1000), report mean ± CI, and flag any build whose simulated value deviates from the analytic score by more
   than a set tolerance (this is our version of WFOpt's `oa` check, but with a tight tolerance and per-build).
7. **Time budget and cancellation**: run as an anytime algorithm with a shared best-so-far (lock-free or `Mutex<TopK>`),
   stream progress and improvements to the UI (SSE/WebSocket), allow abort.

### 12.3 Matching WFOpt (parity mode)
For the accuracy audit, implement a "WFOpt-compatible" mode that reproduces its observable choices: same exclusions
(status-duration-only, next-shot regex, prohibited list defaults, noFaction default on, maxCond default on), same slot
groups, same metrics (`dps` = median per-second bucket of the queue, `hit_infight` = mean per-hit total, TTK banding),
optionally the same `prefer` bias. The goal is: our optimiser's best build should score ≥ WFOpt's best build under our
own honest metric on the same scenario; the headless API (`WFOpt.runFull`, `setLogSink`, `evalAgg`) can be used to
collect WFOpt's results for a set of test weapons.

### 12.4 Doing better than WFOpt
* Constraints WFOpt lacks: capacity/drain with polarities, forma count, Orokin reactor, aura/stance capacity bonus, mod rank
  selection (allow under-ranked mods to fit capacity), per-slot locks, must-include / must-exclude mods and elements.
* User-chosen objective: sustained DPS (with reloads), burst DPS (no reload), TTK against a full enemy (all layers), per-hit,
  or a weighted multi-attack objective (e.g. normal + heavy); report a small Pareto set (e.g. DPS vs TTK vs ammo use).
* Conditional mods: explicit per-condition toggles and uptime fractions instead of one global "conditions met" switch;
  report sensitivity to conditions as WFOpt's `yd` does.
* Riven: optionally search a riven's stat roll space (choose the best 2-3 positives + negative) as a separate mode.
* Reporting: ranked alternatives per element mix (WFOpt's "best for each damage mix" table is useful), leave-one-out
  contribution of each mod, marginal value of the next best replacement, and the stochastic CI of the final numbers.

### 12.5 Caching
* Mod stat vectors and slot pools: computed once per (weapon, attack, context).
* Score memo: `DashMap<CanonicalBuildHash, Score>` (or per-thread caches merged at the end) keyed by the canonical build
  **plus a context hash** (enemy, level, buffs, strength, headshot, condition toggles, window), so no flush discipline is
  needed (avoids Q12).
* Partial-sum caching in branch-and-bound (bucket totals of the prefix), which is what makes deltas O(1).
* Simulation results keyed by (build, context, seed block) for the validation stage.
