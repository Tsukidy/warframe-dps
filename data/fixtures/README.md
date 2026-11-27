# Test fixtures

Owner: T03 (work-breakdown.md §8). A later task that needs more fixture records adds
`mini/extra/<task-id>.json` instead of editing the base files (work-breakdown.md §2.2).

| Path | Content |
|---|---|
| `mini/` | hand-written bundle in the `data/generated/` JSON layout (pipeline.md §8.1); `WFD_DATA_DIR=data/fixtures/mini` points `wfd-data` at it, and without `WFD_DATA_DIR` and without `data/generated/` `wfd-data/build.rs` embeds it with a `cargo:warning` (AD-11; contract review T02c-R5) |
| `mini/SOURCES.md` | the citation of every record (wiki module + revision, Public Export, spec section) |
| `mini/COVERAGE.md` | every row of domain-model.md §17 and the records that encode it |
| `scenarios/*.json` | `CalcInput` documents on the mini bundle (global stable keys) |
| `scenarios/real/*.json` | `CalcInput` documents whose keys are session-local and carried by uniqueName in `x` (AD-18), so they decode on the real bundle (CLI and smoke checks of the shipped binary use these, contract review API-18; optimizer request fixtures live with their crate, `crates/wfd-opt/tests/fixtures/`, T33) |
| `reference_weapons.toml` | reference weapon per item class for `xtask effect-smoke` (uniqueNames of the real bundle) |
| `tools/` | the Python scripts that write and audit the fixture (see "Tools") |

`crates/wfd-model/tests/fixtures.rs` (`cargo test -p wfd-model --test fixtures`) checks all of it.

## File shapes of `mini/`

Every file is the readable (JSON) serde form of the named `wfd-model` type; pretty-printed, keys sorted, records sorted
by uniqueName within a file (pipeline.md §8.1; the lists of `modular.json` too, `ChamberGrip.grip` indexing the
sorted `grips`); `stable_keys.json` sorted by key. The test `record_files_are_sorted` checks it. Effects keep their
binding fields (`bind`, `bind_max`, `bind_duration`, `bind_chance`) as `data/generated/` does: provenance that no
consumer outside the data build reads (domain-model.md §15.1; contract review DC-27). `manifest.json` `data_version`
starts with `fixture-`, so `xtask dist` (T39) refuses to package a fixture bundle (AD-11; checked by
`contract_review_encodings`).

| File | Type |
|---|---|
| `manifest.json` | `Manifest` |
| `weapons/{primary,secondary,melee,archgun,archmelee,exalted,companion,modular}.json` | `Vec<Weapon>` |
| `items/{weapon_mod,weapon_arcane,frame_mod,aura,frame_arcane,shard,stance,incarnon,buff,custom}.json` | `Vec<Item>` |
| `sets.json`, `stances.json`, `combo_tables.json`, `frames.json` | `Vec<SetDef>`, `Vec<Stance>`, `Vec<ComboTable>`, `Vec<Frame>` |
| `modular.json` | `ModularTables` |
| `enemies/<wiki sub-module>.json` | `Vec<Enemy>` (`corpus`, `grineer`, `infestation`, `orokin`, `themurmur`) |
| `rules.json` | `Rules` |
| `names/en.json` | `BTreeMap<NameKey, String>` (the `"en"` entry of `NameTable`) |
| `stable_keys.json` | `Vec<(StableKey, UniqueName)>` as `[[key, uniqueName], …]` |
| `extra/<task>.json` | an object with any of `weapons`, `items`, `enemies`, `sets`, `frames`, `combo_tables`, `stances` (record lists), `names` (map), `stable_keys` (pairs), `rules` (an extension of `rules.json`) and `modular` (an extension of `modular.json`); see "Extension files" |

### Extension files (`extra/<task>.json`)

A later task that needs more fixture data adds `mini/extra/<task-id>.json` and never edits the base files
(work-breakdown.md §2.2). The loader reads `extra/*.json` in file-name order after the base files and applies each
file in this order (the same rule for T04's `wfd-data` loader and T18's emitter; CONTRACTS.md "T03 fix"):

1. Record lists (`weapons`, `items`, `enemies`, `sets`, `frames`, `combo_tables`, `stances`) are appended after the
   records loaded so far. Their dense ids continue the sequence (`id == position`), so records of an extension file
   are sorted among themselves but come after every base record.
2. `names` entries are added; a key that already exists is an error. `stable_keys` pairs are appended; a duplicate
   key or uniqueName is an error (the stable-key test).
3. `rules` and `modular` are deep-merged into the readable JSON form of `Rules` and `ModularTables`: objects merge
   key by key, arrays are concatenated (base first), a key that is absent or `null` in the base takes the
   extension's value, and every other collision (a scalar, an existing map entry such as a `game_constants` key) is
   an error. An extension adds rows and keys; it never overrides one. Only the keyed list tables below and the
   citation lists (`…sources`) append; any other list (the fixed-size `status` table, the `co_default` flags and
   `off_with` lists) cannot be extended; `game_constants` gains keys; scalars (`riven_formula.disposition`, the
   `co_default` rules) cannot change. After the merge no two rows of a keyed list share a key, so a second row for
   an existing key (a second T-01 descriptor, a second `nikana` 9-A row) is an error, not an override. The keys
   (`LIST_KEYS` in `tests/fixtures.rs`; a list with two keys checks each):

   | List | Key |
   |---|---|
   | `factions` | `id`; `slug` |
   | `scaling` | `name` |
   | `mult_groups`, `special_dr`, `riven_traits`, `toggles` | `id` |
   | `slot_layouts` | `class` |
   | `body_parts.faction_heads` | each entry of `factions` (a faction belongs to one row) |
   | `riven_formula.layouts` | (`positives`, `negatives`) |
   | `melee.heavy_class` | `class` |
   | `melee.finishers` | (`class`, `weapons`): the two `exalted_weapon` rows of the real 9-E table differ by `weapons` |
   | `melee.stance_co` | (`stance`, `class`) |
   | `melee.attack_triggers` | (`weapons`, `trigger`, `attack`, `attack_idx`) |
   | `melee.heavy_kappa`, `melee.combo_specials` | the uniqueName (first element of the pair) |
   | `helminth.restricted` | the uniqueName |
   | `helminth.hosts` | `frame` |
   | `categories` | `bit`; `name` |
   | `tags` | `id`; `name` |
   | `modular.{chambers,grips,loaders,strikes,links}` | `unique_name`; `key` |

   Appended rows that carry a dense id (`Faction.id`, `TagDef.id`, `CategoryDef.bit`) continue the base sequence; a
   new modular grip is appended, so it does not shift `ChamberGrip.grip` of the base chambers.
4. The merged result must deserialize into the model types; every check of `tests/fixtures.rs` applies to it.
5. `manifest.coverage` is derived: the loader adds the extension's items to it (count per `ItemKind::slug()` per
   `EffectStatus`), so it always equals the counts of the merged item list (test
   `manifest_coverage_counts_the_items`).

`tests/fixtures.rs` (`apply_extra`, `deep_merge`, test `extension_files_merge_by_the_documented_rule`) is the
reference implementation.

Dense ids (`WeaponId`, `ItemId`, `EnemyId`, …) are the positions in the concatenation of the files in the order of the
table (weapons and items in the listed file order, enemies in file-name order), then `extra/*.json` in file-name
order; every record carries its id and the test checks `id == position`. The `spec/` tables, `REPORT.md` and
`LICENSE` of §8.1 are build outputs of T18 and are not part of the fixture.

Stable keys follow server-api.md §4 where it names them (Braton Prime 1204, the 00 §3.8 mods 3301-3308, Lancer
9001); the rest are blocks: weapons 1200-1601, items 3301-3726, modular parts 6100-6500, frames 8000-8003, enemies
9001-9007. A kitgun chamber and a zaw strike share the uniqueName and the key of their weapon record.

## Contents

- Weapons (19): Braton Prime (Normal Attack of 00 §3.8), Kuva Karak (progenitor), Paris (uncharged and charged, CO
  row 59), Acceltra (projectile + radial child), Amprex (beam), Arca Plasmor (multiplying CO), Rubico (sniper combo,
  zoom levels), Gorgon (spool), Lenz (delayed explosion), Torid (lingering Poison Cloud), Stahlta (embedded child),
  Furis with its Incarnon form and every perk of its evolutions, Nikana Prime (Blind Justice stance, stanceless
  Nikana table, synthesized heavy), Machete (synthesized heavy with σ_1 = 1.2, TV-07-10), Glaive (throws as melee
  projectiles, heavy recall explosion), Telos Boltace (UniqueToWeapon table), Glory (exalted, Strength model 1),
  Catchmoon kitgun chamber, Balla zaw strike. Incarnon attacks of Braton Prime, Paris, Gorgon and Torid are left out
  (the Furis carries the Incarnon case).
- Items (129): the eight 00 §3.8 mods and the rows of domain-model.md §17 (mods, arcanes, shards, the Blind Justice
  stance, Incarnon perks of Furis (all eight), Braton, Laetum, Phenmor and Kunai, buffs of T-13 … T-19 and the
  catalogue with their source kind `ability_source`, the 27 custom stats of 04 §10: the inspiration's 26 and
  heatInheritFaction). Set mods: Vigilante (6), Gladiator (3 of 6), Sacrificial (2), Nira (3), Synth Charge, Hunter
  Munitions. The Incarnon perks of Braton, Laetum, Phenmor and Kunai (families 1-4) are engine-only fixtures: no
  fixture weapon owns their family, so they test effect lowering, not equipping. Items whose encoding the DSL cannot
  express exactly, or whose values are not fully sourced, are `draft` with a note (Sacrificial Steel and Pressure,
  Stormburst, Headcracker); `mini/SOURCES.md` gives the reason per item. Melee Doughty (`doughty_rank`, ruling C-37)
  and Theorem Demulcent (four per-element values, C-38) follow the contract review and are verified; pass 3 made
  Prelude of Might the rows `cd_base_cond` / `cd_base_cond_thr` (02 E-2.5), custom p.k the `sc_vuln` group
  `custom_sc_vuln`, and added Braton's Critical Parallel (CDM0+) and the Mandonel field (`sc_mul`).
- Enemies (7): Lancer, Heavy Gunner (Eximus scenario, TV-05-05), Kuva Trokarian (overguard, Kuva Grineer),
  Corrupted Heavy Gunner, Elite Crewman (shields, TV-05-08), Hemocyte (boss with attenuation), Rogue Culverin
  (critical promotion part).
- Frames: Jade (two aura slots), Excalibur, Rhino (Helminth host of Roar, 09 E-9.32 H3) and Saryn (frame passive
  SD +0.25, `Frame.passives`, which T-16 Toxic Lash implies; 04 E-4.14).
- Rules: the subset the scenarios need (status table complete; factions, scaling, multiplier groups incl. `slam`,
  slot layouts with element positions, CO defaults, body parts, Hemocyte special DR, four riven traits and the riven
  formula, a descriptor for every toggle a scenario sets, melee rows 9-A / 9-D for Nikana, Tonfa, Dagger and 9-A for
  Machete, the Helminth rule, the game constants). The full curated rule tables are T03b's and T03c's
  (`data/curated/rules/`); every toggle descriptor, multiplier group, slot layout, body-part row, Helminth row, riven
  trait, special-DR definition and game constant of `rules.json` is a copy of its curated row (test
  `rule_rows_follow_the_curated_tables`; contract review T03b-R6, DC-23, DC-19).

## Scenarios

| File | Case |
|---|---|
| `braton_prime_lancer_l100_sp.json` | spec 00 §3.8 (Braton Prime, 8 mods, Lancer L 100 Steel Path, body) |
| `real/braton_prime_lancer_l100_sp.json` | the same document with session-local keys and `x` uniqueNames |
| `furis_incarnon_corrupted_heavy_gunner_l1000_sp.json` | Furis with Genesis, perks Haven Foray (`inc2` 0), Extended Volley (`inc3` 1) and Prelude of Might (`inc4` 1), the riven of domain-model.md §16.6, Secondary Shiver |
| `nikana_prime_blind_justice_light_loop.json` | melee stance scenario: Blind Justice, `stance_combo = neutral`, `melee_plan = light_loop` |
| `kuva_karak_toxin_kuva_trokarian_l100.json` | Kuva weapon with a 0.60 Toxin progenitor vs a Kuva Grineer overguard unit |
| `amprex_elite_crewman_l200_sp.json` | shielded Corpus unit (TV-05-08) with a beam |
| `rubico_eximus_heavy_gunner_l58_overguard.json` | overguard Eximus (TV-05-05), headshots, zoom level 1 (6x, T-38), Primed Chamber, Primary Deadhead |
| `gorgon_heavy_gunner_l150_strip_viral.json` | armored target with T-07 external strip 0.5, T-06 external Viral 10, Terrify (ability strip) and an ally-cast Roar (`wfd:buff/ally_roar`, not the T-13 buff) at caster Strength 2.0 (T-33) |

Toggle numbering: the documents store field names, not toggle numbers, so T01d's renumbering of `TOGGLE_MAP` to the
final spec (CONTRACTS.md T01d) did not change them. The scenarios change T-03 (`hit_location`), T-06
(`external_viral`), T-07 (`armor_strip`), T-30 (`melee_plan`) and T-38 (`zoom_level`) from their defaults, and T-33
through a buff `caster` (the Gorgon scenario's ally Roar at Strength 2.0), plus the document parts of T-02, T-13,
T-24; `rules.json` has a descriptor for each, and one for T-37 (the caster of the T-13 Roar, whose default `ally` no
scenario changes). The test `toggle_descriptors_match_the_toggle_map_and_the_scenarios` checks every `toggles.*`
field that differs from its default and every buff `caster` against the descriptors (a caster other than the buff's
`default_caster` needs T-37 for the T-13 buff and T-33 otherwise; an ally Strength or Duration other than 1.0 needs
T-33), and every descriptor's `field` against `TOGGLE_MAP`. T01d changed T-22: `combine_groups` is now `null` (the profile default) in the Braton Prime scenarios,
where `[]` would now mean "every attack its own unit"; and the T-13 Roar buff carries `default_caster = "ally"`
(T-37).

## Tools

`tools/` holds the scripts that wrote the fixture JSON, so it can be regenerated and audited from the repository
(Python 3, standard library only; run with `python3 -I`):

| Script | Purpose |
|---|---|
| `fixture_io.py` | loads the bundle with symbolic cross references, saves it re-sorted, renumbered and in the readable layout above, and recomputes `manifest.coverage` |
| `coverage.py` | computes `manifest.coverage` from the item files; `--write` stores it, otherwise it checks it |
| `lua_table.py`, `stance_e70.py` | Module:Stances/data entries → `ComboTable` (07 E-7.0, `addSharedCombos`) |
| `check_combo_tables.py` | audit: recomputes every table of `combo_tables.json` and its combo names from the cached Module:Stances/data revision of `manifest.json` and reports any difference (needs `research/external`, CLAUDE.md; `--external DIR` or `WFD_EXTERNAL`) |
| `history/review_fixes_2026_11_27.py` | the data edits of the first T03 fix run, applied to the fixture of e3a0049 (reproduces d0b7dd0's JSON; needs `research/external` for the Machete and Glaive tables) |
| `history/review_fixes_2026_11_27_round2.py` | the data edits of the second fix run on top of d0b7dd0 (Haven Foray's Overshields line, the T-33 and T-37 descriptors, `spec_rev` and `data_version`) |
| `history/contract_review_2026_11_27.py` | the data edits of the whole-contract review, pass 1 (CONTRACTS.md "Contract review fixes (data)"): Melee Doughty, Theorem Demulcent, Melee Animosity, Crepuscular, Piercing Navigator, Toxic Lash, `ability_source`, heatInheritFaction, frames Rhino and Saryn, and `rules.json` rows copied from the curated rule tables; needs `data/curated/rules/` |
| `history/contract_review_pass3_2026_11_27.py` | the data edits of the whole-contract review, pass 3 (CONTRACTS.md "Contract review fixes (pass 3)"): Electric Shield `dot_scaling = false`, custom p.k as the `sc_vuln` group `custom_sc_vuln`, the Mandonel field buff, Prelude of Might as `cd_base_cond` / `cd_base_cond_thr`, the Braton perk Critical Parallel, abilityCombo's T-11 range; needs `data/curated/rules/` |

The `history/` scripts are records of one-time edits (each restores or expects a specific base); new fixture data goes
into `mini/extra/<task>.json` (above). The original T03 run's generator was not kept; its records are audited by
`tests/fixtures.rs` and, for the combo tables, by `check_combo_tables.py`. `SOURCES.md`, `COVERAGE.md` and this file
are edited by hand. `manifest.json` `spec_rev` names the `docs/mechanics` commit the records follow (fc67d67, the
contract-review spec round); `data_version` is bumped whenever the content changes (`fixture-mini.4`).
