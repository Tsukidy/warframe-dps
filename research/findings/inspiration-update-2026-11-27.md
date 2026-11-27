Status: draft

# Inspiration bundle update of 2026-11-27: line map and behaviour changes

Scope: where the inspiration code cited by the spec (`docs/mechanics/*.md`) and by the findings
(`research/findings/inspiration-*.md`, `oracle-feasibility.md`, one citation in `wiki-warframe-side-buffs.md`) now
sits, and which behaviour the spec attributes to the inspiration changed.

Inputs:

| | old (cited by the spec) | new (this file) |
|---|---|---|
| `min.js` sha256 | `5265a03779891aef1b8a2902c0671440fbd53a5d928a898c9bf81e474d363de4` | `7bb618341d34c02a1fc7400b29aef9e2410fa7dd5797615f1cb532a4285393e5` (Last-Modified Fri, 27 Nov 2026 11:15:48 GMT, `FETCHED.json`) |
| `min.pretty.js` sha256 | `7b9e4711068651646fe95c791fe5f7371e5c37ac1daa5bf4abf43ae9d71b3eff` | `ba6cf810bf83d3bdad0507410176691505c3ab00ed4be3caf72149fe366d95c0` (prettier 3.3.3, default options) |
| lines | 22,038 | 22,300 |

New-file lines are cited as `min.pretty.js@7bb6 L…`; old lines as `L…` (the form the spec uses). The old file is not
available in this container, so every statement about the old code rests on what the spec and the findings say it
contained; where they quote it, the quote is reproduced.

Method:

1. Collected every inspiration line citation with a script: 2,706 citations (1,676 distinct ranges) in 18 files
   (556 in the ten `docs/mechanics/` files, 2,150 in the findings). A reference counts as an inspiration citation when
   it follows `JS`, `min.pretty.js` or `min.js` in the same clause, or stands in an `inspiration-*.md` file without a
   nearer wiki, HTML, `index.txt`, JSON or `I:`/`W:`/`M:` marker. Wiki and module line references were excluded.
2. Relocated anchors by content: top-level function definitions (names survive minification), string literals,
   property keys, constants and mod names. 121 of the 175 top-level functions between new L5443 and L15264 match a
   whole-function citation (same start and same end after the offset), which is strong evidence that those functions
   only moved.
3. Derived the piecewise map (section 2), then read every region where the offset changes (section 3).
4. Checked every citation automatically against the map: the citing clause's code tokens (backticked identifiers,
   camelCase or snake_case words, quoted strings, decimals) are searched in the mapped range ±2 lines (section 6).

---

## 1. Summary

- The site's change log has one entry dated 2026.11.27: "Improvements to the build-optimization tool, including fixes
  for several bugs. Changes to the calculation of bonus critical damage (similar to the approach used in Arcane
  Crepuscular)." (section 5). Both halves are visible in the code.
- **Logic change 1, crit multiplier (`getFinalWeaponCritMulti`)**: absolute crit damage `crit_mult_add` (Arcane
  Crepuscular, Tenacious Bond, custom `p.h`) is now added to the crit multiplier **before** tier scaling and before
  the headshot doubling. Melee Doughty (`incr_CM_by_punc_status`) stays after the tiers. Affects D-014 (00, 02),
  the D-014/D-206 compat formula and TV-02-23 (02), and is new evidence for OQ-016 (00). Body crits at tier 1 give
  the same value as before. Section 3.2, 4.1.
- **Logic change 2, Condition Overload count in the queue**: the status-proc base damage of each hit (the DoT base
  `kb`/`fa` of the findings) now uses the number of status types present in that hit's timeline entry
  (`getCountDifferentStatusesFromTimeline`, new function), not the static number of damage types. The direct hit,
  the display panel and the crit-chance input `critFlatChancePerStatuses` keep the static count. Affects D-056
  (00, 04), T-05 `weapon_types` and the OC2 oracle comparisons (08). Section 3.4, 4.2.
- **Optimizer**: rewritten in places (evaluation cache, trust check of the fast estimate, and much of the code after
  old L16073). The parts the spec cites (D-092, D-807, D-808, D-809) are unchanged in behaviour. Section 3.6.
- **Data**: `modsProps` grew by 8 lines (593 → 594 entries) inside the arch-gun mod block (old L2472-2582); which
  entry was added cannot be decided without the old file. No spec citation falls in that block. Section 3.1.
- Everything else moved by a constant offset: 0 (data up to old L2471), +8, +3, +7, +8, +13 (damage engine,
  TTK, UI handlers up to old L15503), then +13 to +262 in the optimizer.
- **Minified local names were renamed** throughout (top-level function names were not). Citations that name a local
  (`ta`, `Ma`, `Jb`, `kb`, `fc`, `Ca`, `fa`, …) must be read by role, not by name: section 7.

---

## 2. Old → new line map

Offset = new − old. "Bracket" rows are regions where the offset changes; their content is in section 3.

| Old lines | Offset | New lines | Anchors (old → new) |
|---|---|---|---|
| 1-2471 | 0 | 1-2471 | L42 Hornet Strike `base: 2.2`; L77/L89 Acuity `5.25`; L165 Sacrificial set; L825 `heavy_crit_mult`; L868 Galvanized Diffusion `multishot: 2.3`; L911/L995/L1111/L2181 `double_crit: 0.05`; L1174 Clip Delegation; L2426 Primed Deadly Efficiency; L2471 Necrophagic Vigor (all `name:` or `action:` lines, unchanged) |
| 2472-2582 | bracket, 0 → +8 | 2472-2590 | `modsProps` arch-gun block (section 3.1) |
| 2583-8313 | +8 | 2591-8321 | L2583 Archgun Ace → L2591; L4548 `random_status_after_status: 0.24` → L4556; L4890 `dbl_mult: 1.5` → L4898; L5158 `flat_crit_chance: 0.45` → L5166; L5434 end of `modsProps` → L5442; L5456 `headShotMultInitial` → L5464; L6677 `restoreSettings` → L6685; L8194 `getFinalWeaponCritChance` → L8202; L8268 `getFinalWeaponCritMulti` → L8276; L8302-8311 Doughty block → L8310-8319; L8312 `n && (d += 100)` → L8320; L8313 `no_headshot_mult` → L8321 |
| 8314-8328 | bracket, +8 → +3 | 8322-8331 | crit value returns rewritten (section 3.2); L8314 `if (100 >= d)` → L8323; L8321-8322 tier `parseInt` / `m++` → L8327-8328 |
| 8329-9672 | +3 | 8332-9675 | L8329 closing brace → L8332; L8330 `getStealthDmgBonus` → L8333; L8668 `rebuildModStatsCache` → L8671; L8843 `setAdditionalMods` → L8846; L9140 `reCalcDisplay` → L9143; L9638 `reCalc` → L9641; L9663-9672 → L9666-9675 |
| 9673-9778 | bracket, +3 → +7 | 9676-9785 | share-link serialisation in `reCalc` (section 3.3) |
| 9779-10826 | +7 | 9786-10833 | L9779 `getSettingsFromUrl` → L9786; L9787 `setQuantize` → L9794; L9822 `setCritDmgToHit` → L9829 (L9826 `CC − 99` → L9833); L9888 `getFinalStatusChance` → L9895; L10311 `getModelQueueDmg` → L10318; L10358-10367 head factors → L10365-10374; L10467-10469 `vuln_status_damage` → L10474-10476; L10679 crit roll → L10686; L10752-10760 Spiteful Defilement → L10759-10767; L10761-10766 Hunter Munitions → L10768-10773; L10821-10825 Cold → L10828-10832; L10826 stealth → L10833 |
| (none) | inserted | 10834 | `const mc = sc ? 0 : getCountDifferentStatusesFromTimeline(d[A]);` (section 3.4) |
| 10827-12019 | +8 | 10835-12027 | L10827-10852 CO block → L10835-10860; L10855-10858 `arrInfo` fields → L10863-10866; L10860-10880 hit multiplier → L10868-10888; L10904 Slash block → L10912; L10988 Toxin → L10996; L11059 Heat → L11067; L11145 Electricity → L11153; L11219 Gas → L11227; L11291 Blast → L11299; L11364 armor pass → L11372; L11882-11956 unique-enemy branches → L11890-11964; L11997 `otherDmg` → L12005; L12015 `getCountDifferentStatuses` → L12023 |
| 12020 | bracket | 12027 | last line of `getCountDifferentStatuses` (old 6 lines, new 5; section 3.5) |
| (none) | inserted | 12028-12033 | `function getCountDifferentStatusesFromTimeline(b)` (section 3.4) |
| 12021-15503 | +13 | 12034-15516 | L12021 `optCheckErrors` → L12034; L12029 `setAdditionalParams` → L12042; L12239 `makeRNDMaskStatusArray` → L12252; L12250 `createDmgQueue` → L12263 (L12251-12278 shot-schedule inputs → L12264-12291; L12527-12547 → L12540-12560); L12845 `setForceProcs` → L12858; L12902 `newElements` → L12915; L13142 `swithTypeUniqueEnemyDR` → L13155; L13193 `addDmgToBase` → L13206; L13262 `getCustomStat` → L13275; L13274 LZString → L13287; L13593 `calcTTKWithStatusUnified` → L13606; L13805 `applyDamageLayer` → L13818; L14717-14728 riven catalogue rows → L14730-14741; L15252 `WFOpt` → L15265; L15314-15348 log and progress → L15327-15361; L15463-15503 snapshot → L15476-15516 |
| 15504-15567 | bracket, +13 → +14 | 15517-15581 | optimizer slot setter (old `U`, new `X`; section 3.6) |
| 15568-15673 | +14 | 15582-15687 | L15568-15571 `__probe` setter → L15582-15585; L15608-15653 stance trial → L15622-15667 |
| 15674-15687 | bracket, +14 → +15 | 15688-15702 | old `na(ctx)` (9 lines) → new `na` (10 lines) and `Ea` |
| 15688-15800 | +15 | 15703-15815 | stratified `isProc` old `Ca` L15688-15704 → `Da` L15703-15719; golden-ratio `Math.random` `ta` L15705-15707 → `ra` L15720-15722; `sa` L15708-15718 → L15723-15733; `Pa` → `Oa` L15734-15737; `Ja` → `Ka` L15738-15740; fresh-hit metric `Qa` L15726-15754 → `Pa` L15741-15769; Xata share `rb` → `tb` L15770-15780; metrics `Hb` L15766-15797 → `Qb` L15781-15812 |
| 15801-15875 | bracket (rewritten) | 15816-15880 | old one-evaluation `Ua` (75 lines) → new `Ua` L15816-15850 plus `$a` L15851-15880 (section 3.6) |
| 15876-15951 | +5 | 15881-15956 | `eb` → `pc` L15881; honest sampling `Wb` → `Rb` L15885-15924; aggregated estimate `la` → `Q` L15925-15956 |
| 15952-16000 | bracket (rewritten) | 15957-16023 | trust check old `oa` (49 lines) → new `vb` (67 lines; section 3.6) |
| 16001-16060 | +23 | 16024-16083 | two-stage confirmation `kb` → `Mb` L16024-16041; score `fa` L16019-16051 → `la` L16042-16074; `Lc` → `Ac` L16075-16083 |
| 16061-22038 | +23 → +262, not constant | 16084-22300 | approximate anchors, each from a single UI id or literal: L16596-16598 `wfoptHeavy` ≈ L16638 (+42); L16818 `base_per_status` ≈ L16897 (+79); L17792-17797 `wfoptElemExclude` ≈ L17916 (+124); L18156-18158 `wfoptBeamW` ≈ L18346 (+190); L21317-21327 `wfoptIncPerks` ≈ L21561 (+244); L21406-21409 `wfoptAllowOneShot` ≈ L21650 (+244); L21500-21506 Autobuild ≈ L21758 (+258); L21575 `wfoptHeadshot` ≈ L21817 (+242); L21610 `wfoptExclBox` ≈ L21847 (+237); L21826 constant `0.6180339887498949` → L22069 (+243); L21861-21869 `next_shot` table ≈ L22123-22129 (+262); L21937-21939 `setLogSink` → L22197-22199 (+260); L22038 → L22300 (+262). Offsets go down as well as up here, so this region was reorganised, not only extended. |

To relocate a citation: find its row, add the offset. In bracket rows use section 3 or the content table in 6.2.

---

## 3. Changed regions

### 3.1 Old L2472-2582: `modsProps` arch-gun mods (data change, entry not identified)

Evidence: L2471 Necrophagic Vigor is at min.pretty.js@7bb6 L2471 (offset 0); L2583 Archgun Ace (cited by `04` as
"inspiration `WITH_COND` (JS L2583)") is at L2591 (+8). The findings count 593 entries in `modsProps`
(`I:model` §1); the new array has 594 (`grep -c '^  {'` over L2-5442). The type counts (12 aura, 36 exilus,
17 frame arcanes, 27 frame mods, 18 shards, 65 stances, 42 weapon arcanes, `I:feat` L95) are unchanged, so the
added entry is an ordinary mod. Entries between new L2477 and L2590: Deadly Efficiency, Venomous Clip, Primed
Venomous Clip, Polar Magazine, Primed Polar Magazine, Combustion Rounds, Primed Combustion Rounds, Electrified
Barrel, Magnetized Cycle, Magazine Extension (twice, identical, L2535-2546), Modified Munitions, Quick Reload, Ammo
Chain, Primed Ammo Chain, Ballista Measure, Automatic Trigger, Shell Rush. None of these entries is 8 lines long,
so the +8 is one new entry plus one or two lines added to other entries. Which entry is new cannot be decided
without the old file. The change log gives no 2026.11.27 data entry.

Classification: data change. Spec impact: none found; no spec or findings citation falls in old L2472-2582.

### 3.2 Old L8314-8328: crit value in `getFinalWeaponCritMulti` (logic change)

New code, min.pretty.js@7bb6 L8301-8331:

```js
  const w =
      getStatValFromMods("crit_mult_add") +
      getStatFromAttackUnique("crit_mult_add", b) +
      getCustomStat("crit_mult_add"),
    x = getStatValFromMods("incr_CM_by_punc_status");
  u = 1 + getCustomStat("crit_mult_mult");
  ...                                        // L8307-8319 Doughty term B, unchanged
  n && (d += 100);                           // L8320
  n = b?.no_headshot_mult ? 1 : headShotMult; // L8321
  b = (b.crit_mult + k) * h + w;             // L8322
  if (100 >= d)
    return !1 === f
      ? { val: (b + B) * u, tier: 1 }
      : { val: (n * (1 + (2 * b - 1)) + B) * u, tier: 1 };
  m = parseInt(d / 100, 10);
  0 != d % 100 && 100 < d && m++;
  return !1 === f
    ? { val: (1 + m * (b - 1) + B) * u, tier: m }
    : { val: (n * (1 + m * (2 * b - 1)) + B) * u, tier: m };
```

With CM = `(crit_mult + flat_critmult [+ prelude]) × (1 + Σcrit_mult + custom + attack unique + heavy_crit_mult)`,
abs = `w`, D = Doughty `B`, x = `u` = 1 + custom `crit_mult_mult`, p = `headShotMult`, k = tier:

| | old (as the spec and findings transcribe it) | new |
|---|---|---|
| body crit | `(1 + k(CM − 1) + abs + D)·x` | `(1 + k(CM + abs − 1) + D)·x` |
| headshot crit | `(p·(1 + k(2·CM − 1)) + abs + D)·x` | `(p·(1 + k(2·(CM + abs) − 1)) + D)·x` |

Old forms: `02` D-014 ("headshot crit value `(p·(1 + k(2·CM − 1)) + abs + B)·x`, body crit
`(1 + k(CM − 1) + abs + B)·x`"), `I:stat` §9.2, `I:status` §5.7. The old function was 62 lines (L8268-8329), the new
one 57 (L8276-8332); the old tier-1 and tier-k returns took 6 lines each (L8315-8320, L8323-8328), the new ones 3
(L8324-8326, L8329-8331), consistent with `+ w` having been a separate summand in each return.

Consequences:

- abs now scales with the tier: body crit at tier k ≥ 2 rises by (k − 1)·abs·x.
- On headshot crits abs is doubled and multiplied by p together with CM: the head part gains p·k·2·abs instead of abs.
- Body crit at tier 1 is unchanged: `(CM + abs + D)·x` in both.
- Doughty (`incr_CM_by_punc_status`) is still added after tiers and outside the head factor.
- Unchanged in the same function: `heavy_crit_mult` in the relative bucket (L8277-8281, L8296-8300; D-706), the
  Prelude of Might condition `d < h.cond` and the `reduce` that keeps the last mod (L8286-8294; D-201), the tier from
  the roll input (L8320, L8327-8328; D-018), no base-CM quantization (D-003), the custom `crit_mult_mult` factor on
  the whole value (D-206).
- Unchanged elsewhere: the display value in `setAdditionalMods` still adds `crit_mult_add` after the multiplication
  (min.pretty.js@7bb6 L8884-8894, old L8879-8892); Cold crit damage is still added to `B[A].crit` after this function
  (L10828-10832, D-015); the Deadhead-type factor (new name `ra`) still multiplies the whole hit (L10370-10374,
  L10871); `setCritDmgToHit` is unchanged (L9829-9859).

The change log entry ("Changes to the calculation of bonus critical damage (similar to the approach used in Arcane
Crepuscular)") matches this change.

### 3.3 Old L9673-9778: share-link serialisation in `reCalc` (undecided; no behaviour difference found)

The block grew by 4 lines (old 106 lines L9673-9778, new 110 lines L9676-9785). The old citations into this block
contradict each other, so the old layout cannot be rebuilt from them: the `o` block "L9699-9718" fits new
L9707-9726 exactly (+8), but `o.md` "L9708" fits L9712 (+4) and the riven block "L9751-9765" fits L9754-9769 (+3).
The new block serialises the same keys the findings list (`I:feat` §15, `I:model` §16): `w e l sp ex j jv z k m`,
`o.{hs s m h md w v r z x l n j d ms g t sc ac}`, `p.{a … aa}` (26 keys, no `p.n`), `r`. The URL write
(`?p=…&n=1`, skipped while `isQueueRun`) and the language links (L9777-9783) are as described.

Classification: cannot decide between reformatting and a code change; no key was added or removed. No spec item
depends on the exact lines (the link format is used by T-21 link import, which reads keys, not lines).

### 3.4 Old L10826/10827 and new L12028-12033: CO count from the timeline (logic change)

New function, min.pretty.js@7bb6 L12028-12033:

```js
function getCountDifferentStatusesFromTimeline(b) {
  return manualCountOfStatuses.is_set
    ? manualCountOfStatuses.val
    : Object.keys(b).length +
        (b.hasOwnProperty(viral) ? 0 : externalVirus.is_set ? 1 : 0);
}
```

Called once, in `getModelQueueDmg`, L10834:

```js
    const mc = sc ? 0 : getCountDifferentStatusesFromTimeline(d[A]);
    if (isMaxCond && !Ac && !be) {
      ...
              currWeaponInitial.attacks[r].damage[Y] *
                (1 + Bc + mc * Kc) *                 // multiplicative CO, L10842
      ...
              currWeaponInitial.attacks[r].damage[Y] * (mc * Kc + Bc + Jc * Ma);   // additive CO, L10846
      0 < cd && !dd && !is_MultiplicativeBasePerStatus &&
        (P = cd * Jc + (ne - cd) * (mc * Kc + Bc + Jc * Ma));                     // flat split, L10850
      vb = P * Q * Oc;                                                             // L10851
      ...
      Mb = P * Q * Oc;                                                             // L10859
```

Here `Kc` = Σ `base_per_status` (L10406-10409), `d[A]` = the status timeline entry of hit A, `vb` / `Mb` = the
crit-path and non-crit-path status-proc base (the findings' `kb` / `fa`, `I:stat` §6.1 rows 4-5, `I:status` §6.1).
`vb` and `Mb` feed every status tick computed for the hit: Slash `0.35 × vb × Q` (L10923-10924) and
`getTickStatusDmg(…, vb | Mb, …)` for Toxin, Heat, Electricity, Gas and Blast (L10998-11307) and the Debilitate /
special branches (L11444, L11625, L11804).

The old code used the static count here: `I:stat` §6.1 row 4 "`Σ_type base × (1+cc+fc·Ka)·Mc·tc` (mult.) or
`Σ base × (fc·Ka + cc + Mc·tc)` (add.)" with `fc` = `getCountDifferentStatuses`; `I:queue` L96 "`fc` = static
status-type count (`getCountDifferentStatuses`, L12015-12020, 0 if immune to all)"; `I:status` L285
"`statusCount` = `getCountDifferentStatuses` (§7)". The CO block grew by exactly the one `mc` line (old
L10827-10852, 26 lines → new L10835-10860 plus L10834).

What the timeline count is (read from `timelineStatus`, L10100-10143): `d[A]` holds one key per status type with at
least one active stack at hit A, and a hit's own procs are counted from its own index (`x[k] += 1`, L10118), so the
count **includes the statuses applied by the hit itself**. `getModelQueueDmg` works on a copy of the timeline that
`extendTimeline` and Melee Duplicate may extend (L10737-10747, L10757). The manual override (`o.ms`) still wins;
external Viral still counts once when no Viral stack is present.

Still static (`getCountDifferentStatuses`, L12023-12027): the per-type hit maps (`reCalcWithNonCritBase` L9239-9240,
`reCalcWithTypeBase` L9377), hence the direct hit damage and the attack panel; `critFlatChancePerStatuses` and the
`statusCount` argument of `getFinalWeaponCritChance` in the queue (`Sc`, L10528-10529, L10668, L10678).

Classification: logic change. Not mentioned in the change log (possibly among "fixes for several bugs").

### 3.5 Old L12015-12020: `getCountDifferentStatuses` (undecided, behaviour as specified)

Old 6 lines (L12015-12020), new 5 lines (L12023-12027). The new body is
`if (manualCountOfStatuses.is_set) return manualCountOfStatuses.val; b = Object.keys(b.damage); return b.length +
(b.includes(viral) ? 0 : externalVirus.is_set ? 1 : 0);`, which is what `I:stat` §6 (L229-232) and `00` D-056 describe
("number of damage types in the attack's modded damage (+1 for external Viral)"). Whether the extra old line was
formatting or code cannot be decided.

### 3.6 Optimizer (old L15504 onward; logic changes, announced)

Read in detail where the spec or the findings rely on it:

- Old L15504-15567 (slot setter `U`, new `X` L15555-15581): copies `name`, `actions`, `tags`, `uncomp`,
  `uncomptag`, `shard`, `info` and `set`, deletes `actions.WITH_COND` for on-kill mods when `noOnKill`, as
  `I:opt` describes; new calls `a === Ld && oa(…)` at L15558 and L15578 (probably the "lock parent element" feature of
  the 2026.11.19 entry). The extra line cannot be placed exactly.
- Old L15674-15687: `na` one line longer; new helper `Ea` (L15698-15702) returns the first empty slot of a list of
  slot groups.
- Old `Ua` L15801-15875 → new `Ua` L15816-15850 (cache lookup) plus `$a` L15851-15880 (one evaluation). The cache key
  (L15819-15841) is now `[attack, defense, queueLength, seed]` + installed mods + Lich element + stance combo + stance
  attack index; it still omits buffs, strength, headshot, `isMaxCond` and custom stats, so `I:opt` Q12 still holds.
  The cache is cleared when it reaches its size limit (L15846-15848).
- Old `oa` L15952-16000 (trust check) → new `vb` L15957-16023: checks every beam leader with 15 paired honest runs,
  logs fast versus honest per candidate, and switches to honest sampling when the honest winner differs from the
  fast winner or when any candidate deviates by more than 20 % (L15984, L16012-16021). The findings describe one build
  and `|dev| > 80` (`I:opt` §4.8, L334). Logic change.
- After old L16073 the order and size of functions differ widely (offsets +23 to +262, not monotone); not read in
  detail. Citations there are listed in 6.3.

Checked and unchanged in behaviour:

| Spec item | Old | New (min.pretty.js@7bb6) | Check |
|---|---|---|---|
| D-809 stratified `isProc` and golden-ratio `Math.random` | L15688-15725 | L15703-15740; constant L22069 | `e = a / (c + 1)`, key `a + ":" + c`, FNV hash seeded by `2654435761 * seed` (L15705-15713); `he = (he + oe) % 1` with `oe = 0.6180339887498949`; reset per evaluation (`Oa`, called in `$a` L15856) |
| D-808 `hit_fresh` / `hit_infight` | L15726-15797 | L15741-15812 | fresh hit = Σ(`totalDmg` + `getStatusDamageSum`) of the earliest `timeM` (L15749-15768); `avgHit` = last `TotalDmg_Queue` value (L15808) |
| D-807 TTK score | L16019-16051 | L16042-16074 | EHP = health + shield + overguard; not killed `−(queueLength + EHP/dps)`; one-shot `0.001 + Math.log1p(hit/EHP)`; sub-millisecond tie-break `(g/(g+e))·(1/speed)·0.3` |
| D-092 no capacity, polarity, Forma, Catalyst | L15252-22038 | L15265-22300 | no match for `polarity`, `forma`, `catalyst` or `capacity` other than `ammoCapacity` |

---

## 4. Affected spec items

### 4.1 Crit multiplier (section 3.2)

| Document, item | Spec statement | New code |
|---|---|---|
| `00` D-014 (register row) | "`crit_mult_add` (Crepuscular, Tenacious Bond) and Melee Doughty added after tier scaling: `1 + k(CDM − 1) + abs` (JS L8314-8328; `I:stat` Q10)"; game column "added to CDM before tiers: `1 + k(CDM − 1 + abs)`"; Toggle "Compat" | `b = (b.crit_mult + k) * h + w;` (L8322), then `1 + m * (b - 1) + B` (L8330): `crit_mult_add` is now before tiers, as in the game column; Doughty `B` is still after tiers |
| `02` D-014 | inspiration: "headshot crit value `(p·(1 + k(2·CM − 1)) + abs + B)·x`, body crit `(1 + k(CM − 1) + abs + B)·x`"; compat k ≥ 1: "`Part_B·M_son·[ (Part_A·c'_k + abs + doughty_unrounded)·(1 + x) + cold_compat ]`" | `(n * (1 + m * (2 * b - 1)) + B) * u` and `(1 + m * (b - 1) + B) * u` with `b` = CM + abs (L8322-8331). The compat bracket must move abs into `c'_k` (CM' + abs), keep Doughty outside |
| `02` D-013, `00` D-013, `05` D-013 | "every headshot crit uses `2·CDM` (JS L8319-8328)" | still `2·b`, but `b` now includes abs (L8326, L8331) |
| `02` D-206 | "multiplies the whole per-hit crit value, including the headshot part and the absolute terms, but not the Cold term added later: `val·(1 + x)` (JS L8293-8328, L10821-10825)" | unchanged in effect: `u` multiplies the whole return (L8325-8331); Cold still added later (L10828-10832). Its compat column refers to the D-014 bracket, which changes |
| `02` TV-02-23 (L1842-1850) | expected compat crit factors 15.0; 19.825; 29.575; 3.9 (headshot crit, CM' 2.0, Part_A 3, k 1, abs 3.0) | by reading the new code: 3·(1 + (2·5.0 − 1)) = **30.0**; 1.3·(30.0 + 0.25) = **39.325**; 1.3·(30.0·1.5 + 0.25) = **58.825**; non-crit 3.9 unchanged. Derived from the code, not from an oracle run (UNVERIFIED by test). The game-profile value in the same row (30.002931118011475) is not inspiration-dependent |
| `02` §4 sources (L369-370) and review note (L1895-1897) | "`JS L8268-8329` (inspiration form, D-014, D-206)"; "the emulation is `Part_B·M_son·[(Part_A·c'_k + abs + doughty)·(1 + x) + cold]`" | location L8276-8332; emulation as in the D-014 row above |
| `00` OQ-016 | "Head-crit doubling of absolute crit damage …; proposed: doubles total CDM" | the inspiration now doubles abs on headshot crits (`2 * b`, `b` = CM + abs, L8326/L8331): the inspiration author's choice now agrees with the proposed answer. Not a game source |
| `04` §5 Arcane Crepuscular row (L1419), EC-04-63 | "CD_abs +3.0 (`1 + k·(CDM − 1 + 3)`)" | inspiration body crits now agree with this form (apart from D-003 quantization) |
| `08` E-8.25 / OC list (L1484) | "crit chance 0 (D-003 touches every crit) \| D-003, D-013, D-014, …" | the D-014 part of every crit oracle comparison needs the new compat form |
| `I:stat` §9.2, Q10; `I:status` §5.7; `I:model` L154-155 (`crit_mult`, `crit_mult_add`); `I:queue` §3.3 L113-120 | `val = (1 + tier × (CD − 1) + add + punc) × mm`; Q10 "`crit_mult_add` and `incr_CM_by_punc_status` are added after tier scaling … not into CM (L8314-8328)" | true only for `incr_CM_by_punc_status` now |

### 4.2 CO count (section 3.4)

| Document, item | Spec statement | New code |
|---|---|---|
| `00` D-056 | "number of damage types in the attack's modded damage (+1 for external Viral), static (`getCountDifferentStatuses`, JS L12015-12020)" | static for the hit and display (L12023-12027, L9239-9240, L9377); **per-hit timeline count** for the status-proc base: `const mc = sc ? 0 : getCountDifferentStatusesFromTimeline(d[A]);` (L10834), used at L10842, L10846, L10850, L10857-10858 |
| `04` D-056 | "number of damage types of the attack, +1 external Viral (JS L12015-12020)" | as above |
| `00` T-05 | "`weapon_types` = number of nonzero damage types of the attack after combination, +1 if T-06 is on and the attack has no Viral" (described as the "inspiration rule") | the inspiration rule is now split: `weapon_types` for direct hits and display, timeline count (own procs included) for DoT bases |
| `08` E-8.25 OC1 (L1419) | "per-type damage … CO at the static count of D-056" | unchanged (display path is static) |
| `08` E-8.25 OC2 | statistical comparisons of "Random queue dps", per-bullet values and TTK with the compat profile | with a CO-type mod (`base_per_status`) the DoT part of every hit now depends on the live status count; a compat emulation needs the timeline count for status bases |
| `I:stat` Q4 (L449), §6.1 row 4 (L262); `I:queue` L96, L327; `I:status` §6.1 (L270-285), Q30 (L410) | Q30: "Condition Overload count = number of weapon damage types, never the statuses really on the target (L12015-12020)"; `I:queue` L327: "CO status count `fc` is static … `critFlatChancePerStatuses` uses the same static count" | no longer true for status bases; still true for direct hits and `critFlatChancePerStatuses` |
| `wiki-warframe-side-buffs.md` L254 | external Viral "counts as one more status type for Condition Overload-type mods (inspiration JS L12015)" | still true for both counts (L12026, L12032) |

### 4.3 Unchanged items checked because they cite a moved or neighbouring line

D-001 (`setQuantize` L9794-9798), D-016 (Hunter Munitions `0.3 * P`, L10768-10773), D-017 (whole-percent rounding,
L10686), D-018 (`h = f - 99`, L9833), D-020 (`makeRNDMaskStatusArray` L12252-12259), D-052 (armor pass from
`currWeapon.health`, L11372-11383), D-067 (Spiteful Defilement `B[A].crit *= 1 + Dc` when stacks < 3,
L10759-10767), D-201 (L8286-8294), D-208 (`double_crit` L9836, L10689), D-210 (`no_headshot_mult` removes both head
factors, L10365-10374), D-211 (`headShotMultInitial = 3`, L5464), D-706 (L8277-8281), D-801 (per-bullet averages,
L12210-12251), D-912 (ten `uncomptag: ["POWER_WEAPON"]` entries, L790-4962), D-010/D-011 (riven `SMITE` weak-point
crit chance and `mult_for_head`, L14735-14741). Special-enemy DR remains dead code: `showEnemy` still sets
`currEnemy.unique = "notset"` (L7698).

---

## 5. Change log and news (step 4)

`index.html` has a "Change log 2026.11.27" button (`<div class="updatedB" tabindex="0">Change log 2026.11.27</div>`,
index.html L433) and a change-log popup whose first entry is (index.html L597-598):

> 2026.11.27 Improvements to the build-optimization tool, including fixes for several bugs. Changes to the
> calculation of bonus critical damage (similar to the approach used in Arcane Crepuscular).

The next entries are 2026.11.20 (arcanes Secondary Cryogenic, Cascadia Accuracy, Primary Obstruct, Primary
Exhilarate, Melee Assimilation), 2026.11.19 ("lock parent element" in the optimizer), 2026.11.18 (minor fixes),
2026.11.17 (TTK calculation). The spec's `I:feat` §1 says the change log it read ran to 2026.11.20, so the
2026.11.27 entry is the only announced change between the two bundles. `guide.html` carries the same list
(guide.html L391-395; its header button still reads "Change log 2024.08.08", guide.html L360). Neither page has
another news or version note for 2026-11-27. `FETCHED.json` records no Last-Modified for index.html or guide.html.

---

## 6. Citation relocation results

### 6.1 Automatic check

Each citation was mapped with section 2 and its code tokens searched in the mapped range ±2 lines:

| Result | Count | Meaning |
|---|---|---|
| pass | 956 | a token from the citing clause is in the mapped range |
| near | 96 | a token is within ±25 lines: the citation pointed at a neighbouring line already in the old file (for example `L2343-2348` for Maiming Strike, whose entry is L2344-2349) |
| no token | 1,042 | the clause has no code token (for example "`JS L10295-10310`" after prose); relocated by offset only |
| fail | 297 | no token within ±25 lines; listed and read: the unmatched tokens are section numbers (`4.1`, `5.3`), prose words, renamed minified names (section 7), or names of callers and callees mentioned in the same clause; no row showed an offset different from section 2. I did not open the code for every one of these rows |
| in a bracket | 315 | handled in 6.2 and 6.3 |

### 6.2 Citations into bracket regions, relocated by content

| Old citation | Cited by | What it says is there | New (min.pretty.js@7bb6) |
|---|---|---|---|
| L8293-8328 | `02` D-206 | p.i factor on the whole crit value | L8301-8331 |
| L8314-8328, L8314-8327 | `00` D-014, `02` D-014, `I:stat` Q10 | abs and Doughty after tiers | L8322-8331 (abs now at L8322) |
| L8314 | `I:status` L253, `02` D-018 | `tier = 1 if cc ≤ 100` | L8323 |
| L8319-8328 | `00`, `02`, `05` D-013; `I:model` OQ3 | headshot crit `p·(1 + tier·(2·CM − 1))` | L8326, L8331 |
| L8321-8322 | `I:status` L253 | `ceil(cc/100)` | L8327-8328 |
| L9638-9778 | `I:agg` L19, `I:stat` L22 | `reCalc` | L9641-9785 |
| L9673-9777 | `I:agg` L38, `I:stat` L39 | build share URL | L9676-9784 |
| L9690-9698, L9691-9696, L9697 | `09` L310, `I:model` L19, L310; `I:stat` L51 | link key `m`, `{mId, pId}` for slots 18-22 | L9698-9706; L9699-9705; L9701 |
| L9699-9718 | `I:model` L423 | link key `o` | L9707-9726 |
| L9708 | `I:stat` L271 | `o.md` | L9712 |
| L9719-9748, L9724-9750 | `I:model` L448, `I:feat` L194 | link key `p.a … p.aa` | L9727-9753 |
| L9751-9765 | `09` L324, `I:model` L394 | link key `r` (riven) | L9754-9769 |
| L9770-9776 | `I:feat` L22 | language links rewritten per recalculation | L9777-9783 |
| L12015-12020 | `00` D-056, `04` D-056, `I:stat`, `I:queue`, `I:status`, `I:feat` L153 | `getCountDifferentStatuses` | L12023-12027 (and the new L12028-12033) |
| L15252-22038 | `09` D-092, `I:feat` L8, `I:opt` L563 | the optimizer | L15265-22300 |
| L15549-15558, L15560, L15561-15565 | `I:model` L22, `I:opt` L566, L108 | optimizer copies `action` into the panel; copies `set`; deletes `WITH_COND` | L15563-15571; L15572; L15573-15577 |
| L15658-15670, L15663-15669 | `I:stat` L479, `I:model` L34 | optimizer sets Incarnon perks | L15672-15684; L15677-15683 (`ya`, L15668-15687) |
| L15674-15682 | `I:opt` L351 | `na(ctx)` | L15688-15697 |
| L15801-15875, L15803-15827, L15835 | `I:opt` L222, L640, L267 | one evaluation, cache key, accumulator reset | `Ua` L15816-15850 and `$a` L15851-15880; key L15819-15841; reset `Hc && Oa()` L15856 |
| L15952-16000, L15962, L15966-15978, L15988 | `I:opt` §4.8 and Q-items, `I:agg` L485-486 | trust check, `|dev| > 80` | `vb` L15957-16023; threshold now 20 % (L15984, L16018); the old lines have no one-to-one counterpart |

### 6.3 Citations not relocated

- **Optimizer after old L16073**: 275 citations (inspiration-optimizer.md 202, inspiration-data-model.md 34,
  inspiration-feature-inventory.md 29, inspiration-dps-aggregation.md 7, inspiration-stat-pipeline.md 2,
  inspiration-queue-model.md 1; none in `docs/mechanics/`). The offset in this region runs from +23 to +262 and is
  not monotone; the anchors in the last row of section 2 give an approximate position (±50 lines between anchors),
  and each citation has to be found by content. Their behaviour was not compared with the findings.
- **Old L2472-2582**: no citation falls inside; the line of any future citation into this block cannot be mapped
  until the added entry is identified.
- **Old L9673-9778 at line precision**: the old citations conflict (3.3); 6.2 gives the content positions instead.
- Excluded as not inspiration citations although the script first matched them: lowercase `wiki:` references in
  the findings (for example `wiki:Hunter_Munitions L2-21`, `wiki:Damage__Puncture_Damage L18`), the abbreviation
  example "`L1234`" in `I:model` L5, and `I:model L161` in `02` L387.

---

## 7. Renamed minified names cited in the spec or findings

Top-level function names did not change. Local variables and parameters were renamed by the minifier; the role is
what identifies them.

| Role | Old name (as cited) | New name and line |
|---|---|---|
| `getModelQueueDmg` parameters | `(b, d, g, p, q, k, t, h = [], w = p, x = p, m = {})` (`I:queue` L28) | `(b, d, f, n, r, k, u, h = [], w = n, x = n, m = {})`, L10318 |
| head multiplier on headshots | `Ca` (`I:feat` L140) | `Da`, L10365-10369 |
| non-crit head factor | `Ma` (`02` D-014, `I:queue`) | `Ia = 1 < B[A].crit ? 1 : Da`, L10784 |
| Deadhead-type factor | `ta` (`02` D-014, D-210) | `ra`, L10370-10374 |
| hit multiplier product | `Jb` (`I:stat` L44) | `Vb`, L10868-10888 |
| status-proc base, crit / non-crit | `kb` / `fa` | `vb` / `Mb`, L10400-10401, L10851, L10859 |
| static CO count in the queue | `fc` | `Sc`, L10528 |
| strength factor | `ya` | `ya`, L10357 (unchanged) |
| optimizer stratified `isProc` / `Math.random` / install / reset / restore | `Ca` / `ta` / `sa` / `Pa` / `Ja` | `Da` / `ra` / `sa` / `Oa` / `Ka`, L15703-15740 |
| optimizer fresh-hit / Xata share / metrics / one evaluation / honest sampling / aggregated estimate / trust check / confirmation / score | `Qa` / `rb` / `Hb` / `Ua` / `Wb` / `la` / `oa` / `kb` / `fa` | `Pa` / `tb` / `Qb` / `Ua`+`$a` / `Rb` / `Q` / `vb` / `Mb` / `la`, L15741-16074 |

---

## 8. Limits

- Without the old file, a change that keeps the line count of a region constant is invisible to the offset map.
  The 121 whole-function matches (Method, step 2) and the automatic check reduce, but do not remove, this risk:
  the CO change (3.4) was found only because it added one line. A line-by-line comparison needs the old `min.js`
  (sha256 `5265a037…`), which cannot be fetched here (web.archive.org is blocked).
- The old formulas in 3.2 and 3.4 are the spec's and the findings' transcriptions, not the old code itself.
- The TV-02-23 values in 4.1 are hand-derived from the new code; they were not produced by the oracle harness.
