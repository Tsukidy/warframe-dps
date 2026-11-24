# Test fixtures

Owner: T03 (work-breakdown.md §8). A later task that needs more fixture records adds
`mini/extra/<task-id>.json` instead of editing the base files (work-breakdown.md §2.2).

| Path | Content |
|---|---|
| `mini/` | hand-written bundle in the `data/generated/` JSON layout (pipeline.md §8.1); `WFD_DATA_DIR=data/fixtures/mini` points `wfd-data` at it |
| `mini/SOURCES.md` | the citation of every record (wiki module + revision, Public Export, spec section) |
| `mini/COVERAGE.md` | every row of domain-model.md §17 and the records that encode it |
| `scenarios/*.json` | `CalcInput` documents on the mini bundle (global stable keys) |
| `scenarios/real/*.json` | `CalcInput` documents whose keys are session-local and carried by uniqueName in `x` (AD-18), so they decode on the real bundle |
| `reference_weapons.toml` | reference weapon per item class for `xtask effect-smoke` (uniqueNames of the real bundle) |

`crates/wfd-model/tests/fixtures.rs` (`cargo test -p wfd-model --test fixtures`) checks all of it.

## File shapes of `mini/`

Every file is the readable (JSON) serde form of the named `wfd-model` type; pretty-printed, keys sorted, records sorted
by uniqueName within a file.

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
| `extra/<task>.json` | an object with any of `weapons`, `items`, `enemies`, `sets`, `frames`, `combo_tables`, `stances` (record lists), `names` (map) and `stable_keys` (pairs) |

Dense ids (`WeaponId`, `ItemId`, `EnemyId`, …) are the positions in the concatenation of the files in the order of the
table (weapons and items in the listed file order, enemies in file-name order), then `extra/*.json` in file-name
order; every record carries its id and the test checks `id == position`. The `spec/` tables, `REPORT.md` and
`LICENSE` of §8.1 are build outputs of T18 and are not part of the fixture.

Stable keys follow server-api.md §4 where it names them (Braton Prime 1204, the 00 §3.8 mods 3301-3308, Lancer
9001); the rest are blocks: weapons 1200-1601, items 3301-3725, modular parts 6100-6500, frames 8000-8001, enemies
9001-9007. A kitgun chamber and a zaw strike share the uniqueName and the key of their weapon record.

## Contents

- Weapons (15): Braton Prime (Normal Attack of 00 §3.8), Kuva Karak (progenitor), Paris (uncharged and charged, CO
  row 59), Acceltra (projectile + radial child), Amprex (beam), Arca Plasmor (multiplying CO), Rubico (sniper combo,
  zoom levels), Gorgon (spool), Lenz (delayed explosion), Furis with its Incarnon form and perks, Nikana Prime
  (Blind Justice stance, stanceless Nikana table, synthesized heavy), Telos Boltace (UniqueToWeapon table), Glory
  (exalted, Strength model 1), Catchmoon kitgun chamber, Balla zaw strike. Incarnon attacks of Braton Prime, Paris
  and Gorgon are left out (the Furis carries the Incarnon case).
- Items (109): the eight 00 §3.8 mods and the rows of domain-model.md §17 (mods, arcanes, shards, the Blind Justice
  stance, Incarnon perks of Furis, Laetum, Phenmor and Kunai, buffs of T-13 … T-19 and the catalogue, the 26 custom
  stats of 04 §10). Set mods: Vigilante (6), Gladiator (3 of 6), Synth Charge, Hunter Munitions.
- Enemies (7): Lancer, Heavy Gunner (Eximus scenario, TV-05-05), Kuva Trokarian (overguard, Kuva Grineer),
  Corrupted Heavy Gunner, Elite Crewman (shields, TV-05-08), Hemocyte (boss with attenuation), Rogue Culverin
  (critical promotion part).
- Frames: Excalibur, Jade (two aura slots).
- Rules: the subset the scenarios need (status table complete; factions, scaling, multiplier groups, slot layouts,
  CO defaults, body parts, Hemocyte special DR, four riven traits and the riven formula, the toggles the scenarios
  use, melee rows 9-A / 9-D for Nikana, Tonfa, Dagger). The full curated rule tables are T03b and T03c.

## Scenarios

| File | Case |
|---|---|
| `braton_prime_lancer_l100_sp.json` | spec 00 §3.8 (Braton Prime, 8 mods, Lancer L 100 Steel Path, body) |
| `real/braton_prime_lancer_l100_sp.json` | the same document with session-local keys and `x` uniqueNames |
| `furis_incarnon_corrupted_heavy_gunner_l1000_sp.json` | Furis with Genesis, perks at tiers 2-4, the riven of domain-model.md §16.6, Secondary Shiver |
| `nikana_prime_blind_justice_light_loop.json` | melee stance scenario: Blind Justice, `stance_combo = neutral`, `melee_plan = light_loop` |
| `kuva_karak_toxin_kuva_trokarian_l100.json` | Kuva weapon with a 0.60 Toxin progenitor vs a Kuva Grineer overguard unit |
| `amprex_elite_crewman_l200_sp.json` | shielded Corpus unit (TV-05-08) with a beam |
| `rubico_eximus_heavy_gunner_l58_overguard.json` | overguard Eximus (TV-05-05), headshots, Primed Chamber, Primary Deadhead |

Toggle numbering: the scenarios use T-01 … T-29 fields only, except the melee plan (`toggles.melee_plan`), which is
T-32 in `TOGGLE_MAP` today and T-30 in the final 00 §6.1; T01d renumbers it (CONTRACTS.md, manager rulings). The
field name, not the number, is stored in the documents, so renumbering does not change these files.
