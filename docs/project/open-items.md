Status: living (project management; updated 2026-11-27)

# Open items at the pause

Everything left unfinished when the work paused, in the order `continuation.md` §5 handles it. Each item names its
source so the next session can verify it before acting.

## 1. Failing checks on the branch head

Resolved 2026-11-27 by T01d and the regeneration for D-056: the full Definition of done passes on the dev branch
(fmt, clippy, tests, spec-sync with 75 Compat flags and 43 toggles, gen-ts, gen-schema, doc-check, lint-engine).

## 2. Contract requests waiting for a manager ruling

Resolved 2026-11-27: every request C-1 … C-30 is ruled (`CONTRACTS.md` "Manager · Rulings on the T01b and T01c
requests" and "Manager · Rulings on T01d"). T01b's duplicate C-17 … C-21 were renumbered C-26 … C-30; the next free
number is C-31.

## 3. T03 review issues (fix run started 2026-11-27, `W0-frontend-fixtures`)

T03 (mini fixtures) was merged at its implementation commit; its review found the 15 issues below and the fix agent
was stopped before it finished. Run the fix as its own `impl-phase.js` task (`id: "T03"`, `mode: "main"`,
`review: true`, a note pointing here) after T01d, since some fixes depend on the renumbered toggles.

1. **critical** — `data/fixtures/mini/combo_tables.json (every strike, e.g. line 17-21); crates/wfd-model/tests/fixtures.rs:818-819`
   - Problem: Strike.phys is written in IMP, PUN, SLA order. The frozen contract orders it SLA, IMP, PUN. A Slash bonus is therefore stored in the PUN slot and a Puncture bonus in the IMP slot. The acceptance test asserts the wrong layout, which hides the error.
   - Evidence: crates/wfd-model/src/weapon.rs:780-781: `/// S^phys_t = factor − 1 for SLA, IMP, PUN (E-7.0 step 5), in that order. pub phys: [f64; 3]`. domain-model.md:253 says the same.

Blind Justice Guiding Light input 1 has SlashMultiplier [2] (Stances_data), but the fixture stores phys [0.0, 0.0, 1.0]. Stanceless Dagger input 0 has PunctureMultiplier [2] and is stored as [0.0, 1.0, 0.0].

The test at fixtures.rs:818-819 asserts `(3.0, [0.0, 0.0, 1.0])` under the comment 'S^phys_SLA = 1.0'. TV-07-08 (S^phys_SLA = 1.0 on Guiding Light 1) will be computed as a Puncture bonus by T08b/T13.
   - Fix: Rewrite every strike's phys as [SLA, IMP, PUN] (Guiding Light 1 becomes [1.0, 0.0, 0.0]). Change the test to index by the contract order, and add one assertion on a Puncture-only strike (Dagger input 0, expected [0, 0, 1.0]).

2. **critical** — `data/fixtures/scenarios/furis_incarnon_corrupted_heavy_gunner_l1000_sp.json:65,73; data/fixtures/mini/weapons/secondary.json:196-208`
   - Problem: The Furis scenario selects perk index 1 in slots inc3 and inc4. Each Furis tier in the bundle holds only one perk, so both picks point at perks that do not exist. Only Haven Foray resolves. Extended Volley and Prelude of Might (the D-014 'base add' case and the magazine case the scenario is meant to exercise) are never picked.
   - Evidence: build.rs:429: `/// inc<t>: index into IncarnonDef.tiers[t-1].` Fixture tiers are [[], [61], [60], [62], []]:
- inc3 reads tiers[2] = [60] at index 1, which is out of range.
- inc4 reads tiers[3] = [62] at index 1, which is out of range.

The scenario test (fixtures.rs:536-611, `every_scenario_decodes_and_resolves`) checks only that StableKeys exist. It never checks Perk indices against tiers, so the bug passes.
   - Fix: Use index 0 in inc3 and inc4, or put the real per-tier perk lists in tiers. Extend the scenario test so every SlotContent::Perk index is < tiers[t-1].len() and every slot's item kind fits its SlotId.

3. **major** — `data/fixtures/mini/weapons/primary.json:1248,1261 (Rubico zoom levels)`
   - Problem: Rubico's zoom critical-damage bonus is encoded as cd_abs. The spec puts sniper zoom crit damage in ΣCD_rel. The Rubico overguard scenario will produce wrong crit multipliers whenever the zoom level is active (aiming is on by default and all_max picks the highest zoom level, D-406).
   - Evidence: 02 §4.3 bucket table (02-crit-status-multishot.md:388): 'ΣCD_rel | … sniper zoom crit damage (Rubico etc.) …'.

EC-02-17 (02:1201-1202): Lanka's zoom is flat after mods, and 'other sniper zoom bonuses are relative'.

Fixture: `"stat": "cd_abs", "value": 0.35` and `0.5`, with no UNVERIFIED mark.
   - Fix: Use stat cd_rel for both zoom levels and cite 02 §4.3 and EC-02-17.

4. **major** — `data/fixtures/mini/items/incarnon.json:427 (wfd:perk/phenmor/spiteful_defilement)`
   - Problem: Spiteful Defilement is encoded as cd_rel. The spec has settled on ΣCD_abs (OQ-406 closed). With Vital Sense this gives 6.402 instead of 5.401 (TV-02-01).
   - Evidence: 04 row at 04-conditional-bonuses-and-buffs.md:1641: '`sum_cd_abs` +1.00 … register D-067 (amended 2026-11-24) and `02` §4.3 put it in ΣCD_abs … (OQ-406 closed)'. 02:390 lists Spiteful Defilement under ΣCD_abs.

The fixture follows the stale domain-model.md:1456 row. work-breakdown §2.4 says the spec wins over the architecture.
   - Fix: Use stat cd_abs and cite D-067 (amended) and 02 §4.3. Record the stale domain-model §16 row for T41/T01d.

5. **major** — `data/fixtures/mini/items/weapon_arcane.json:20-31 (Melee Doughty)`
   - Problem: Melee Doughty is encoded as cd_rel × floor(SC_PUN / 0.10). The spec defines a different bucket and formula: ΣCD_abs with doughty = min(50, round1((r_PUN/Σr)·100·SC·0.1·d_rank)), applied only to the Puncture-bearing part. The rounding (floor vs round-half-up to 0.1), the input (sc_of.PUN vs Puncture share × weapon SC), the cap and the bucket all differ.
   - Evidence: 02 E-2.5a (02:400-405) and ΣCD_abs (02:390: 'Melee Doughty (`doughty`, bucket UNVERIFIED, OQ-209)').

04 row 1336: 'Melee Doughty | CD_abs `doughty = min(50, round1(…))` … applies only to the Puncture-bearing part'. TV-02-14 and TV-04-22 give +9.7 for Destreza Prime.

The fixture again copies domain-model.md:1463 instead of the spec.
   - Fix: Encode the E-2.5a form in ΣCD_abs and cite 02 E-2.5a and 04 §5.4. If the DSL cannot express it, mark the item `draft` and record the DSL gap in CONTRACTS.md, rather than shipping a `verified` record with the wrong formula.

6. **major** — `data/fixtures/mini/items/weapon_arcane.json:217 (Primary Deadhead)`
   - Problem: Primary Deadhead's stack timer is set to shared_lose_all and marked 'UNVERIFIED: stack timer kind'. The spec names it explicitly as shared_lose_one (24 s, full decay 72 s). The simulated and MC modes would drop all three stacks at once.
   - Evidence: 04 stack-timer table (04:1028): '`shared_lose_one` | … Primary and Secondary Deadhead (24 s, full decay 72 s) … (`W:Primary_Deadhead` line 32 …)'. 04 row 1305: 'Primary / Secondary Deadhead | A +1.20 per stack × 3 … 24 s, `shared_lose_one`'.
   - Fix: Use stack_timer shared_lose_one, remove the UNVERIFIED note and cite 04 §4.2 (line 1028).

7. **major** — `data/fixtures/mini/COVERAGE.md; crates/wfd-model/tests/fixtures.rs:438-509`
   - Problem: Several domain-model.md §17 rows are only partly covered. The test accepts a row as soon as it names one existing record, so the gaps pass. T17's effects_coverage.rs needs a fixture for every mechanic in each row. Missing:
- the σ≠1 heavy and 9-A breakdown case (Machete, TV-07-10): every heavy in the bundle has σ = [1, 1];
- lingering children (Torid cloud): no attack has `lingering`;
- set `member_scale` (Sacrificial): all four sets have member_scale null;
- the `mult = "slam"` group (Nira's set): no slam multiplier group in rules.json mult_groups;
- param-typed vulnerability (Theorem Contagion);
- Arcane Arachne on a glaive throw: no glaive weapon in the bundle;
- `strip_ability`, `ammo_eff` and `charge_rate`: no item uses them.
   - Evidence: domain-model.md:1488: 'Every row has a fixture item in data/fixtures/mini/'. The §17 fixture column names Machete heavy (TV-07-10), Torid cloud, Sacrificial, Nira's set, Theorem Contagion, 'Arcane Arachne on a glaive throw', and encodings strip_ability, ammo_eff, charge_rate.

`grep` finds no member_scale other than null in sets.json, no 'slam' group in mult_groups, and no strip_ability, ammo_eff or charge_rate in items/*.json. COVERAGE.md's own header admits that the Torid cloud is not included.
   - Fix: Add the missing records: a Machete or another σ≠1 class with its 9-A row, Torid, Sacrificial set mods, a slam multiplier item with its mult group, Theorem Contagion with a type param, a glaive, and items for strip_ability, ammo_eff and charge_rate. Make the coverage test check each encoding named in the §17 Encoding column, not only one uniqueName per row.

8. **major** — `data/fixtures/mini/items/buff.json (wfd:buff/magus_accelerant effects)`
   - Problem: Magus Accelerant is fixed at one stack (`per = ["stack"]`, `stacks.max = 1`) with no parameter for the number of applications. The spec and architecture make it a user count with no limit. The documented 3-application example cannot be expressed.
   - Evidence: 04 row 1402: '+0.65 per application, Heat / Puncture only, permanent, no limit | (c) count of applications' (Ignis Wraith example `(1 + 3 × 0.65)`). domain-model.md:1469: 'value 0.65 per application (per = ["stack"], stacks = user count, class (c))'.
   - Fix: Add a count parameter, bind the stack maximum to it (or use an unbounded maximum with a user count), and cite 04 §5.5.

9. **major** — `data/fixtures/README.md (extra/<task>.json rule); crates/wfd-model/tests/fixtures.rs:61-74 (struct Extra)`
   - Problem: The extra/<task>.json merge format can append only weapons, items, enemies, sets, frames, combo_tables, stances, names and stable_keys. It cannot add rules.json entries or modular parts. rules.json holds only a subset:
- 6 factions and 5 scaling families (no Techrot);
- heavy_class for 3 melee classes;
- 9 toggles;
- 1 special_dr.
A later task (T07, T08b, T14) that needs more rule rows would have to edit the T03-owned rules.json, which breaks the ownership scheme of work-breakdown §2.2.
   - Evidence: work-breakdown §2.2: 'a later task that needs more fixture entries adds a new file data/fixtures/mini/extra/<task-id>.json (the loader merges extra/*.json)'. Extra has `deny_unknown_fields` and no `rules` or `modular` field.
   - Fix: Define rule-table extension in the extra format: per-table append for the list tables, a merge for the map tables, and modular parts. Document the merge order in README.md and CONTRACTS.md so T04 and T18 implement the same rule.

10. **minor** — `data/fixtures/mini/COVERAGE.md:3-8; data/fixtures/mini/rules.json toggles`
   - Problem: COVERAGE.md says the toggle-only parts of rows (T-06 external Viral, T-07 external strip) live in scenarios and rules.json. Neither is true: no scenario sets armor_strip or external_viral, and rules.json has no T-06 or T-07 descriptor. rules.json also has no descriptor for melee_plan, which the Nikana scenario uses, although README says rules.json holds 'the toggles the scenarios use'.
   - Evidence: rules.json toggles hold only T-01, T-02, T-03, T-09, T-13, T-15, T-20, T-24 and T-25. A grep of scenarios/*.json for a non-zero armor_strip or a non-null external_viral finds nothing.
   - Fix: Add a scenario with T-06 and T-07 set, plus their descriptors (and the proposed melee-plan descriptor), or correct the claims in COVERAGE.md and README.md.

11. **minor** — `data/fixtures/mini/frames.json; crates/wfd-model/tests/fixtures.rs:191`
   - Problem: frames.json is not sorted by uniqueName: Excalibur comes before Choir (Jade). pipeline.md §8.1 and the README both require that order, and dense FrameIds depend on it. The test comment says records are sorted, but nothing checks it.
   - Evidence: frames.json ids: 0 = /Lotus/Powersuits/Excalibur/Excalibur, 1 = /Lotus/Powersuits/Choir/Choir. pipeline.md §8.1: 'records sorted by `uniqueName`'.
   - Fix: Sort the frames (Jade id 0, Excalibur id 1) and add a sort assertion for every record file.

12. **minor** — `data/fixtures/reference_weapons.toml; crates/wfd-model/tests/fixtures.rs:851-865`
   - Problem: The file says its keys are 'the classes of domain-model.md WeaponClass plus the modular, companion and archwing classes'. Six WeaponClass members have no reference weapon: speargun, shotgun_sidearm, amp, crossbow, dual_shotguns, tome. The test checks only that there are at least 10 entries starting with /Lotus/; it does not check the keys against WeaponClass.
   - Evidence: enums.rs:305-326 defines WeaponClass with 20 members; the TOML has 17 keys and none of the six above.
   - Fix: Add reference weapons for the missing classes, or state which classes are out of scope. Make the test check that every key is a WeaponClass name or one of the documented extras.

13. **minor** — `data/fixtures/mini/items/weapon_arcane.json (Primary Plated Round stacks)`
   - Problem: Plated Round's duration is fixed at 10 s. The spec says the duration depends on rank (3, 4, 5, 6, 8, 10 s), so a build with a lower-rank arcane gets the wrong duration.
   - Evidence: 04 row 1313: 'the value does not depend on rank, the duration does (3 / 4 / 5 / 6 / 8 / 10 s)'. Fixture: "stacks": {"duration": 10.0, "max": 1} with no bind_duration.
   - Fix: Bind the duration to the rank table.

14. **minor** — `data/fixtures/mini/manifest.json sources; SOURCES.md:163`
   - Problem: Module:Enemies/data/orokin and themurmur are cited with revision 'fetched-2026-11-23' instead of a wiki revision id or timestamp. These fixtures cannot be traced to a revision the way pipeline.md expects.
   - Evidence: manifest.json: {"id": "Module:Enemies/data/orokin", "revision": "fetched-2026-11-23"}.
   - Fix: Record the actual revid or timestamp from the cache index.

15. **minor** — `crates/wfd-model/tests/fixtures.rs; CONTRACTS.md`
   - Problem: The commit edits two paths outside the task's ownership (`data/fixtures/**`): a new test file in the T01a-owned crates/wfd-model/** tree, and CONTRACTS.md. Both are recorded in CONTRACTS.md, and the acceptance command needs the test file, so this is process noise rather than a conflict. The ownership map should name the file's owner.
   - Evidence: `git show --stat f0c505f` lists CONTRACTS.md and crates/wfd-model/tests/fixtures.rs besides data/fixtures/**. work-breakdown §8 gives crates/wfd-model/** to T01a.
   - Fix: Ask the manager to add `crates/wfd-model/tests/fixtures.rs` to T03's row of the §8 map.

## 4. Items left undone by T01b, T01c and T03 (their own reports)

- T01b: none beyond the contract requests; the merge conflict it predicted was resolved as it prescribed (commit
  "Merge T01b").
- T01c: MechanicKind variants are not in the lint-engine exemption list (wfd-model has no list of them).
- T03: Balla (zaw) carries placeholder melee data; Incarnon attacks of Braton Prime, Paris and Gorgon, and the §17
  example weapons Torid, a glaive and Machete are not in the fixture (COVERAGE.md names substitutes); `rules.json` is a
  subset (full tables are T03b/T03c); two enemy modules are cited as "fetched-2026-11-23" without a revision id; the
  fixture generator was a scratch tool and is not committed.

## 4b. Items T01d left for later owners (CONTRACTS.md "Review fixes (T01d, second pass)")

- T19 / T13: E-4.9 gives no rate for `while_firing` (Flux Overdrive's 04 row gives a per-mode value); the compile rule
  of domain-model.md §12.2 for it is open. Validator checks: `Slot` elements only in slots with `element_order`;
  `spot` exactly on `Spot`-stage members with params of the right kinds.
- T41: optimizer.md l.59 (`CapacityRule` precedence of server-api.md §8.1); optimizer.md and testing.md still describe
  Overwhelming Attrition as a reader of "instances with no crit and no status" (now the trigger rate of `OnPlainHit`).
- domain-model.md §16.13 Melee Doughty row still has the old form (02 E-2.5a is the spec); the T03 fix run decides
  whether the DSL can express it.
- Findings that describe the inspiration's pre-2026-11-27 behaviour (I:stat Q10 and §9.2, I:status §5.7, Q30 and §6.1,
  I:queue L96 and L327, I:model L154-155, I:opt §4.8) are historical; the update file supersedes them for D-014,
  D-056 and the optimizer trust check.
- Contract review pass 3 (CONTRACTS.md "Contract review fixes (pass 3)"; every item of
  `docs/project/inputs/contract-review-recheck.json` was applied there, nothing was deferred). Follow-ups it leaves:
  - T21 (or whoever first adds the Athodai to the mini fixture through `data/fixtures/mini/extra/<task>.json`): the
    §17 token `Cond::Attack` has no fixture user since Prelude of Might became the rows `cd_base_cond` /
    `cd_base_cond_thr` (ENG-R7); add Pain Points (`Cond::Attack(AttackPred::Roles)` at its gain event, DC-10, 04 §5.3)
    with its weapon, then drop the `Cond::Attack` gap in `crates/wfd-model/tests/fixtures.rs`
    (`every_coverage_encoding_is_exercised`, gap count back to 3) and in `data/fixtures/mini/COVERAGE.md`.
  - T25: `rules/mult_groups.toml` now has the `sc_vuln` groups `custom_sc_vuln`, `pageflight` and `dante_passive`
    (DC-12); Dante's passive still needs its user flag (default on when Dante is the frame, 04 §5.6).
  - T01b / T41: `cargo doc -p wfd-model --no-deps` reports three older broken intra-doc links (`api/mod.rs` L17
    `codegen::typescript_files`; `api/opt.rs` `dps_sustained[auto]` twice, unescaped brackets); not in any
    acceptance command, left as they are.

- Fixture data bug (pass-3 tail fixer): `wfd:buff/nourish` has a VIR line without `dot_scaling = false`, which 03 §9.3
  and the `effect.rs` doc require. No numeric effect today (Viral has no DoT). Fix with a fixture history script in the
  next task that edits the fixture (T03 owner), and in T25's real data.

## 5. Spec

- Done 2026-11-27: the converge round of 2026-11-24 was rechecked (two fix rounds), its six-item tail fixed, and the
  spec updated for the 2026-11-27 inspiration bundle; every final recheck was clean.
- Open questions marked UNVERIFIED (`OQ-` items in each document) need in-game testing; they go into the final report.
  OQ-016 (does the headshot doubling also double flat crit damage?) gained weight: the inspiration now doubles it.

## 6. research/external rebuild kit

Done 2026-11-27: `research/tools/` (README, rebuild script, 571 wiki pins, manifest of 716 files), tested from an
empty directory. Not rebuildable: the earlier inspiration bundle (web.archive.org is blocked here), `cache/poc/`
outputs (regenerable once the data pipeline exists), the WFCD source clone (GitHub API blocked).
