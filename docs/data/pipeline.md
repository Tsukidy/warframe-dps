Status: reviewed (architecture)

# Data pipeline

This document defines where game data comes from, how it is fetched, parsed, joined, enriched with curated effects,
validated, packed into the bundle the binary embeds, and refreshed after a game update. The implementation is the
crate `wfd-datagen` (library + CLI `wfd-datagen`); the server's in-app refresh calls the same library (feature
`refresh`). Types are those of `docs/architecture/domain-model.md`. Findings referenced: `DS:wiki`
(`data-source-wiki-modules.md`), `DS:wfcd` (`data-source-wfcd.md`), `DS:pe`
(`data-source-public-export-and-connector.md`).

Contents: 1 Sources · 2 Stages and commands · 3 Fetching and caching · 4 Parsing · 5 Joining, deriving, spec tables ·
6 Effects: tiers and curated tables · 7 Validation and auto-patching · 8 Output and bundle · 9 Refresh after a game
update · 10 Licences and attribution · 11 Module and file inventory

---

## 1. Sources

| Source | Used for (authoritative fields) | Not used for | Licence |
|---|---|---|---|
| wiki.warframe.com Lua data modules (`DS:wiki`) | weapon attacks (per-attack damage map per pellet, crit, status, fire rate, multishot, falloff, radius, forced procs, charge, burst, spool, ammo cost, Incarnon form attacks, `AttackParentIndex`), melee fields, enemies (1,019: faction, base level, pools, body-part prose, `ProcResists`, Eximus fields), `DamageTypes/data` faction table, `Stances/data` combos, `Modular/data` kitgun/zaw parts, `Mods/data` and `Arcane/data` identity (description text, `Incompatible`, `Type`, `UpgradeTypes`, set membership), `Mods/itemCompatibility/data`, `Warframes/data` (126 frames: polarities, aura polarity, exilus polarity, abilities by name) | effect values (prose only), frame base stats (not damage inputs) | CC BY-NC-SA 3.0 (`meta=siteinfo&siprop=rightsinfo`) |
| DE Public Export (`DS:pe`) | canonical `uniqueName` of every DE item, `buildLabel` freshness, top-level weapon stats (cross-check only), trigger, mastery, polarities, mod drain/rarity/max rank/`compatName`, `ExportModSet`, arcane catalogue, riven `upgradeEntries` base values per class, warframe `uniqueName`s and Helminth abilities (`ExportWarframes`, `ExportAbilities`), image paths (`ExportManifest` `textureLocation`) | attacks, enemies, effects (display text only) | none stated (`DS:pe` open question 1): used for identifiers and numbers that WFCD (MIT) already redistributes |
| WFCD warframe-items, pinned tag (`DS:wfcd`) | English names, descriptions, image names, other locales later, per-rank `levelStats` text (rank checks of curated effects), catalogue cross-check | attacks (lossy), enemies (stale damage model), effects | MIT |
| Spec tables (`docs/mechanics/`) | tables the spec marks as normative or ingestable: CO catalogue (04 §2.14), status summary (03 §4.1), faction modifiers (05 §5.1), scaling coefficients (05 §6.1); ingested by the developer build into `data/generated/spec/*.json`, which is the form the binary embeds for in-app refreshes (§5.2) | — | project |
| Curated tables (`data/curated/`) | every effect that is not a simple stat line, rules not in the spec tables (09 slot layouts, multiplier groups, melee hand tables 9-A to 9-E of 07 §4.4, Helminth rules), overrides (06 §4.3, 05 §4.5), aliases | — | project |

The inspiration bundle is never a data source (CLAUDE.md). Its names appear only in `data/curated/aliases/inspiration.toml`
for link import.

---

## 2. Stages and commands

```text
fetch    network → .cache/wfd-sources/ (the only stage that touches the network)
parse    cache → typed wire records per source               (mlua sandbox for wiki modules)
join     wire records → one catalogue keyed by uniqueName     (precedence table §5.1)
derive   attack keys and roles, parent links, groups, triggers, compatibility, stable keys
effects  tier 1 stat lines, tier 2 spec tables, tier 3 curated TOML → EffectDef lists, overrides merged
validate schema, references, bindings, ranks, ranges, cross-source, coverage, anchors
emit     data/generated/*.json (canonical) + REPORT.md + report.json; optional bundle .bin
```

CLI (`crates/wfd-datagen/src/main.rs`, clap):

```text
wfd-datagen fetch   [--offline] [--update-lock] [--only wiki|pe|wfcd]   # refresh the cache; --update-lock bumps data/sources.lock.toml
wfd-datagen fetch --from-lock                           # restore exactly the locked inputs into an empty cache (§3.3)
wfd-datagen build   [--offline] [--out data/generated] [--bundle out.bin]
wfd-datagen check                                       # parse + join + effects + validate, no emit; CI runs this
wfd-datagen diff <old dir|bundle> <new dir|bundle>      # field-level diff ("Furis/normal_attack cc 0.05 → 0.07")
wfd-datagen report                                      # coverage, conflicts, stale entries (markdown + json)
wfd-datagen spec-tables [--check]                       # (re)ingest spec tables into data/generated/spec/
wfd-datagen curate todo [--kind weapon_mod] [--limit 50]
wfd-datagen curate scaffold <name|uniqueName>           # writes a draft TOML prefilled from upstream text
wfd-datagen curate rebind <file>...                      # recompute upstream text, sha256 and binding check
wfd-datagen curate render <file>                         # canonical English rendering next to the upstream text
```

Library entry points (the server uses these):

```rust
// crates/wfd-datagen/src/lib.rs
pub struct PipelineConfig { pub cache_dir: PathBuf, pub curated: InputTree, pub spec: SpecSource,
                            pub offline: bool, pub lock: LockMode /* Pinned | Latest | FromLock */ }
/// A directory on disk (developer, `--curated DIR`) or a tree embedded in the binary with `include_dir!` (in-app
/// refresh: `data/curated/` and `data/generated/spec/`, embedded by wfd-server behind feature `refresh`).
pub enum InputTree { Dir(PathBuf), Embedded(&'static include_dir::Dir<'static>) }
/// Developer builds read the spec Markdown (`Docs`, §5.2) and write `data/generated/spec/*.json`; the in-app refresh
/// reads those already-ingested tables (`Ingested`) and never needs `docs/`.
pub enum SpecSource { Docs(PathBuf), Ingested(InputTree) }
pub fn fetch(cfg: &PipelineConfig, progress: &dyn Progress, cancel: &AtomicBool) -> Result<FetchReport, DatagenError>;
pub fn build(cfg: &PipelineConfig, progress: &dyn Progress, cancel: &AtomicBool) -> Result<BuildOutput, DatagenError>;
pub struct BuildOutput { pub bundle: Bundle, pub json: GeneratedJson, pub report: Report }
pub fn diff(old: &Bundle, new: &Bundle) -> DataDiff;
pub trait Progress: Sync { fn step(&self, step: PipelineStep, done: u32, total: u32); fn message(&self, m: Message); }
```

---

## 3. Fetching and caching

### 3.1 The polite client

`wfd-datagen::fetch::PoliteClient` follows the pattern of warframe-connector (`DS:pe` part 2):

| Rule | Value |
|---|---|
| Concurrency | one request at a time per host; at most one request per second per host |
| Retries | only on HTTP 429, 503, 509 and connection errors; at most 3; honour `Retry-After` up to 60 s, else exponential back-off 2-4-8 s |
| Not retried | every other 4xx; a 404 is cached as a negative result for the run |
| User-Agent | `warframe-dps-datagen/<version> (local non-commercial tool)`; no personal data, no contact address |
| MediaWiki | `maxlag=5` on every API call; batched `prop=revisions` with ≤ 50 titles per request |
| Conditional requests | Public Export index by `ETag` (`If-None-Match`); wiki modules by `revid`; WFCD by pinned tag and SHA-256 |
| Hosts allowed | `wiki.warframe.com` (Lua modules), `content.warframe.com` (Public Export manifests and item images), `raw.githubusercontent.com` (WFCD files at a tag), `registry.npmjs.org` (WFCD version discovery only); nothing else. Checked from this environment on 2026-11-23: all four answer; `cdn.warframestat.us` (WFCD's image CDN) does not, so it is not used |
| Timeouts | connect 10 s, read 60 s |
| Cancellation | checked between requests (the server's refresh can be cancelled) |

### 3.2 Per-source procedure

**wiki modules** (`DS:wiki` §1): one `prop=revisions&rvprop=ids|timestamp` query for all module titles of §11
tells which `revid`s changed (2 requests for about 60 titles). Changed modules are fetched with
`action=query&prop=revisions&rvprop=content|timestamp|ids&rvslots=main&formatversion=2&maxlag=5` in batches of
≤ 50 titles. If a batch hits the API result-size limit (untested, `DS:wiki` §1), the client halves the batch.
Each module is stored as `wiki/<Module_Name>@<revid>.lua`.

**Public Export** (`DS:pe` §1.1): conditional GET of `https://content.warframe.com/PublicExport/index_en.txt.lzma`
(ETag). Decompress the legacy LZMA-alone file with `lzma-rs`, split on CRLF, and fetch each needed manifest with the
index line verbatim (no `\r`, no re-encoding of `+`): `https://content.warframe.com/PublicExport/Manifest/<line>`.
Manifest URLs are content-addressed: cached forever by hash. The live `buildLabel` is read from the index metadata
and recorded.

**WFCD** (`DS:wfcd` §1): raw files at the pinned tag,
`https://raw.githubusercontent.com/WFCD/warframe-items/<tag>/data/json/<Category>.json` (Primary, Secondary, Melee,
Arch-Gun, Arch-Melee, SentinelWeapons, Warframes, Mods, Arcanes, Misc); each verified by SHA-256 from `sources.lock.toml`.
`--update-lock` and the in-app check find the newest release through the npm registry
(`GET https://registry.npmjs.org/@wfcd/items` → `dist-tags.latest`, e.g. `1.1276.51`; the git tag is `v` + version),
so no GitHub API host is needed.

**Images** (§10): `GET https://content.warframe.com/PublicExport/<textureLocation>` with the `textureLocation` of the
item's `ExportManifest` entry (verified on 2026-11-23: the Braton icon returns 200). The bundle stores the
`textureLocation`; WFCD `imageName` is kept only as a display fallback name.

### 3.3 Cache layout and lock file

```text
.cache/wfd-sources/                     (gitignored; --cache-dir overrides; the server uses <data_dir>/cache)
  index.json                            url → { sha256, etag, revid, fetched_at, status }
  wiki/Module_Weapons_data_primary@123456.lua
  pe/index_en.txt.lzma  pe/ExportWeapons_en.json!<hash>
  wfcd/<tag>/Primary.json
```

```toml
# data/sources.lock.toml — pins the exact inputs of the committed data/generated/
[wiki]
"Module:Weapons/data/primary" = { revid = 123456, timestamp = "2026-11-14T08:12:00Z" }
[pe]
build_label = "2026.10.28.09.54"
index_etag = "…"
manifests = { "ExportWeapons_en.json" = "…hash…", "ExportUpgrades_en.json" = "…" }
[wfcd]
tag = "v1.1276.51"
files = { "Primary.json" = "sha256:…" }
```

`wfd-datagen build --offline` uses exactly the locked inputs, so the committed JSON is reproducible.
`wfd-datagen fetch --from-lock` fills an empty cache with exactly those inputs: wiki modules by revision id
(`action=query&prop=revisions&revids=…&rvprop=content|ids|timestamp&rvslots=main`), Public Export manifests by their
locked content hash, WFCD files at the locked tag, each checked against the lock (revid, hash, SHA-256). It is the
first step of every fresh worktree or CI job that builds data (work-breakdown.md §2.3), so no task depends on the
gitignored `research/external/cache/`. If an input is no longer served (an old Public Export hash removed by DE),
the command fails with key `data.lock_input_gone` and the hint to run `fetch --update-lock`; data tasks then commit the
new lock with the regenerated data.

---

## 4. Parsing

### 4.1 Wiki Lua modules (mlua)

Port of the proof of concept in `research/external/cache/poc/lua2json/` (`DS:wiki` §9):

- `mlua` 0.12 with features `lua51`, `vendored`, `send`. Only the `math`, `string` and `table` libraries are loaded;
  `io`, `os`, `package`, `debug`, `load`, `loadstring`, `loadfile`, `dofile` are absent (the last four come with the
  Lua 5.1 base library that mlua always opens and are removed; request T02c-R7), and `require` is replaced by the stub
  below.
- Stubs: `mw.loadData(title)` and `require(title)` return the already-evaluated module of that title from the batch
  (needed by `DamageTypes/data` and `Stances/data`, which run code at load); `mw.site` and `mw.text` minimal stubs.
- Modules are untrusted code: instruction limit via `set_hook` every 10^6 instructions, abort after 10^9; memory limit
  `Lua::set_memory_limit(256 MiB)`; a module that trips a limit fails the build with its title.
- Conversion to `serde_json::Value`: tables with keys exactly 1..n become arrays; other tables become objects with
  string keys; empty tables become `{}` and the typed wire struct resolves the ambiguity; `math.huge` becomes
  `f64::INFINITY` (written as the string `"Infinity"` in JSON output); integers stay integers.
- Typed wire structs (`wfd-datagen/src/wiki/*.rs`) use `#[serde(default)]` and `#[serde(alias)]` for the known typos
  (`Hitscan`, `Ammocost`, `ForcedProc`, `EximusSHield`, `IsSlilent`; `DS:wiki` §11). Entries with
  `_IgnoreEntry = true` are skipped. Unknown fields are kept in a `rest: BTreeMap` and counted in the report, so a new
  upstream field is noticed.

### 4.2 Public Export and WFCD

- PE manifests: strict `serde_json` with a lenient fallback that strips raw control characters (older exports needed
  it, `DS:pe` §1.1). Float32 noise (`0.050000001`) is rounded to 6 significant digits on import.
- WFCD: plain JSON. `levelStats` lines keep DE colour tokens (`<DT_FIRE_COLOR>`); the stat-line importer (§6.2)
  maps them to damage types.

---

## 5. Joining, deriving, spec tables

### 5.1 Join precedence

Everything is joined on `uniqueName` (`DS:pe` §1.8); names are never join keys. The precedence table lives in code
(`join::PRECEDENCE`) and is printed into `REPORT.md`:

| Field group | First source | Fallback | Conflict handling |
|---|---|---|---|
| identity, slot, trigger, mastery, polarities, drain, max rank, rarity, mod sets | PE | wiki | report only |
| top-level weapon crit, status, fire rate, magazine, reload | wiki (per attack) | PE | relative difference > 1e-4 reported; PE sums attacks for some weapons (Kuva Bramma), allow-list in `data/validation/base.toml` |
| attacks, forced procs, falloff, charge, burst, spool, Incarnon form attacks | wiki | — | — |
| names, descriptions, images, locales | WFCD | wiki `Name` | — |
| enemies (all fields) | wiki | — | WFCD enemies never used (`DS:wfcd` §6) |
| warframes (identity, polarities, aura and exilus polarity, abilities) | PE `ExportWarframes` (uniqueName, polarities) | wiki `Warframes/data` | polarity differences reported; aura slot count and Helminth host from curated rules |
| image path | PE `ExportManifest` `textureLocation` | WFCD `imageName` (name only) | — |
| faction damage-type table | spec 05 §5.1 (ingested) | wiki `DamageTypes/data` | a difference is an error unless listed in `data/curated/rules/factions.toml` with a citation |
| mod compatibility, incompatible lists | wiki `Type` + `Mods/itemCompatibility/data` + `Incompatible` | PE `compatName` | union of both directions (D-093) |
| riven trait base values | PE `upgradeEntries` | — | — |
| effects | curated > tier 2 > tier 1 | — | a curated entry replaces automatic effects of that item entirely |

Unmatched records are listed with a reason class (beast weapons, innate upgrades, Antique arcanes, railjack). Expected
match rates are asserted (testing.md §4): WFCD→PE 668/668 weapons; wiki weapons→PE ≥ 825/872; wiki mods→PE ≥
1,451/1,635.

### 5.2 Spec-table ingestion

`data/spec_tables.toml` lists the tables the pipeline reads from `docs/mechanics/`, by file, heading and expected
column header. A header mismatch is an error, so a reformatted spec table is noticed, never misread:

```toml
[[table]]
id = "co_catalogue"
file = "docs/mechanics/04-conditional-bonuses-and-buffs.md"
heading = "2.14 Per-attack catalogue (ingestable)"
columns = ["#", "Class", "Weapon(s)", "Attack", "Data attack", "Behaviour", "B listed", "B_co wiki", "ρ_co", "co_ref", "B_co^abs", "Flags"]

[[table]]
id = "co_defaults"
file = "docs/mechanics/04-conditional-bonuses-and-buffs.md"
heading = "2.5 Per-attack data and the default rule"

[[table]]
id = "status_summary"
file = "docs/mechanics/03-status-effects.md"
heading = "4.1 Summary table (normative; the engine transcribes this)"

[[table]]
id = "faction_modifiers"
file = "docs/mechanics/05-enemies.md"
heading = "5.1 Table (post-U36)"

[[table]]
id = "scaling_coefficients"
file = "docs/mechanics/05-enemies.md"
heading = "6.1 Curve coefficients"
```

Each table has a typed row parser in `wfd-datagen/src/spec_tables/`. CO catalogue rows expand family rows
("Braton/Mk1/Prime/Vandal") to every listed weapon, map wiki attack names to bundle attacks, and become `CoSpec` on the
attack; rows that do not map are listed in the report and fixed in `data/curated/overrides/co_names.toml` (names only,
never numbers). Attacks without a row get the default rule of 04 §2.5, whose table D1-D6 is the fifth ingested table
(`co_defaults` → `Rules.co_default`; contract review DC-19), so no default is hand-copied into code or curated data. The
game constants that no spec table carries (Steel Path factor, Eximus steps; `Rules.game_constants`) come from
`data/curated/rules/game_constants.toml` (§6.1), each row with its spec source. The generated form is written to
`data/generated/spec/*.json` for review; the spec stays the only place these numbers are written by hand. The bundle
manifest records the hash of the ingested tables (`spec_rev`). These JSON files are also the refresh inputs:
`wfd-server` embeds `data/generated/spec/` (with `data/curated/` and `data/spec_tables.toml`, through its `build.rs`,
§9.2) behind feature `refresh`, and the in-app pipeline runs with `SpecSource::Ingested(InputTree::Embedded(..))`, so a
refreshed bundle keeps exactly the tier-2 data of the release (AD-13). `wfd-datagen spec-tables --check` fails when the
JSON is stale against the Markdown, so the embedded form cannot drift from the spec.

### 5.3 Derive

| Step | Rule |
|---|---|
| Attack key | `AttackKey { slug: slugify(AttackName), idx }`; duplicate slugs get `_2`, `_3` |
| Attack roles | from `AttackName`, `ShotType`, `AttackParentIndex`, Incarnon markers and melee fields, by the rule table `data/curated/rules/attack_roles.toml` (regex on attack names → roles); roles no rule can derive come from `overrides/attacks/` (`AttackRecord.roles`), e.g. `stun_gated` on Zymos "Headshot Explosion" (06 E-6.24; contract review M-9) |
| Parent links and groups | `AttackParentIndex` with the override table `data/curated/overrides/attacks/*.toml` (`AttackOverrideFile`: the 02 §4.10 attack records and the groups) for the inconsistent cases (`DS:wiki` §11; 06 §9.1); `comb` groups for T-22 from children of one parent plus `overrides` |
| Trigger | normalisation of the 33 wiki strings (06 §4.2) and per-attack overrides |
| Hand tables of 06 §4.3 | spool cadence, Kohm family, beam ramp start, per-shell reload, semi-cap overrides, bow doubling (`bow`, mapped onto `WeaponTraits::BOW`), fixed charge time, no-reload weapons, exalted ammo mode, reload-triggered attacks, whole-magazine attacks, sniper combo exclusions, weapon and attack passives: `data/curated/overrides/weapons/<slot>/<slug>.toml` (`WeaponOverrideFile`), where `<slot>` is the name of the bundle weapon file the weapon lands in (`primary`, `secondary`, `melee`, `archgun`, `archmelee`, `exalted`, `companion`, `modular`; §8.1; contract review N-9) |
| Exalted Strength and zoom gates | hand-authored, no data source states them (01 §4.3, 06 §9.1): `WeaponOverrideFile.exalted` → `Weapon.exalted` (model and per-rank values of 01 Table 4.3-A, every `Class = "Exalted Weapon"` entry, under `overrides/weapons/exalted/`), `AttackFireOverride.exalted_s` → `Attack.exalted_s` (an attack's own model-1 table: Noctua's active attack), `AttackRecord.zoom_level` (`overrides/attacks/`) → `Weapon.zoom[n].enables` and the role `zoom_only` (Komorex "3.5x Zoom Radial Attack" at its 3.5× level, EC-06-13); each value cites the Table 4.3-A or 06 §9.1 row in the file's `sources`, and T19 checks them against those rows where it can (manager ruling N-1, path (a)) |
| Primary Compression | the per-attack effectiveness and mode of `W:Primary_Compression` (`Multiplies`, `Adds`, "Doesn't Work") → `Attack.compression_eff` and `Attack.compression_mode` (04 §3.1, TV-04-15; contract review DC-8) |
| Enemy body parts | parse `Multis` prose into `BodyPart`s; head-crit capability by the rules of 05 §4.3; overrides in `data/curated/overrides/enemies/*.toml` (`EnemyOverrideFile`: part rows, `replace_parts`, `default_part`); renamed wiki enemies through `aliases/enemies.toml` (`EnemyAliasFile`) |
| Enemy special DR | 05 §4.5 table in `data/curated/rules/special_dr.toml`; Protea is the record `corpus:Protea Specter` (the Deadlock Protocol quest boss, in the `attenuation = unknown` row); a row without a module record (the data gaps with `enemies = []`: Kuva Liches, Sisters of Parvos and their Hounds, the Technocyte Coda, whose spawn form the Coda Duets are, and Lich Thralls) is reported, not an error, and attached when the record appears (request T03b-R3; contract review NEW-CR-3) |
| Compatibility | wiki `Type` → categories via `Mods/itemCompatibility/data`; compat tags; `uncomptag`; Semi-only trigger requirement |
| Modular parts | kitgun chambers/grips/loaders, zaw strikes/grips/links from `Modular/data` (assembled at runtime, `wfd-engine::assemble`) |
| Stance normalisation | 07 E-7.0 on every input of every combo of the evaluated `Stances/data` (with `addSharedCombos` run in Lua): defaults, padding, proc holes, extra procs, strike expansion by `Hits[e]`, `Types[e]` → `StrikeType`; one data-report line per correction; Conclave-only tables dropped. Runs after the join, with the map from the module's stance names to the `StanceId`s of `Bundle.stances` (`normalise_stances(raw, stance_ids, report)`; API-22). Output: `Bundle.combo_tables` (stance, stanceless class and UniqueToWeapon tables, `from_weapon` kept) and `Stance { table }` (domain-model.md §4) |
| Combo table links | per melee weapon: `MeleeData.stanceless` = the stanceless table of its `Class`, `MeleeData.unique` = the UniqueToWeapon table matched by name or `Family` (07 E-7.0b); the per-scenario choice between unique, stance and stanceless combos is made by the engine (E-7.0b needs the equipped stance) |
| Heavy attacks | per melee weapon with a heavy: the synthesized heavy `Attack` (roles `HEAVY`, `synthesized = HeavyFrom { normal }`, 07 §9.1) and its `HeavyDef`: hit lists H_j of E-7.40 (unique override, else class table 9-A with σ, else the 9-A breakdown) and κ_w of E-7.41 (needs Public Export data; checked against table 9-B, a mismatch is a report line: `heavy_def(…, report)`, API-22). The engine resolves the attack's damage per scenario |
| Melee triggers and specials | table 9-C rows attached to their weapons (`MeleeData.triggers`), combo specials and innate Initial Combo (`MeleeData.specials`), ability-combo flag (E-7.44) |
| Frames | `Warframes/data` joined to `ExportWarframes` on uniqueName (names when the wiki lacks a uniqueName, reported): polarities, aura polarity, exilus polarity, aura slot count (2 for Jade from `rules/frames.toml`), abilities, Prime/Umbra `variant_of`, Helminth host from `rules/helminth.toml` (09 E-9.32), frame passives from `rules/frames.toml` `[[frame.passive]]` (`Frame.passives`, 04 §5.11; DC-14) → `Bundle.frames` |
| Stable keys | `data/generated/stable_keys.json` is append-only and global: new uniqueNames get the next integer below `LOCAL_KEY_BASE` (2^31); removed ones keep their key and are listed as `retired`; a key is never reused. The bundle carries both lists: `Bundle.stable_keys` (every entry names a record) and `Bundle.retired_keys` (disjoint; a document key there is `doc.item_retired`, dropped with a warning, a key in neither is `doc.key_unknown`; API-5). Only the developer build writes the file. An in-app refresh (§9.2) never renumbers: it starts from both lists of the embedded bundle (`StableKeySource::Embedded`), uniqueNames already there keep their global key (a re-added retired one included), new ones get session-local keys ≥ 2^31 in the order of their uniqueName, and every document that uses a local key carries its uniqueName (`CalcInput.x`, AD-18) so another machine or a later release resolves it by name |

---

## 6. Effects: tiers and curated tables

### 6.1 Curated directory layout

```text
data/curated/
  weapon_mod/<family>/<slug>.toml      # family = rifle, shotgun, pistol, sniper, bow, primary, melee, archgun, exalted, augment, companion
  weapon_arcane/<class>/<slug>.toml
  frame_mod/<slug>.toml  aura/<slug>.toml  frame_arcane/<slug>.toml
  shard/<color>/<slug>.toml
  stance/<slug>.toml                   # stance CO ratios and stance-specific corrections (combos come from the wiki)
  set/<slug>.toml                      # set bonus effects
  incarnon/<slot>/<family>.toml        # one file per Incarnon family (tiers, perks, variant values); slot = primary, secondary, melee
  buff/<group>/<slug>.toml             # ability, focus, companion, squad, debuff, environment
  custom/<slug>.toml                   # the 27 custom stats of spec 00 §6.3 (26 plus one companion input)
  rules/
    mult_groups.toml  slot_layout.toml  riven_traits.toml  riven_formula.toml  body_parts.toml
    special_dr.toml  toggles.toml  factions.toml  categories.toml  helminth.toml  frames.toml  game_constants.toml
    attack_roles.toml  stat_lines.toml
    melee/heavy_class.toml  melee/heavy_kappa.toml  melee/attack_triggers.toml  melee/stance_co.toml
    melee/finishers.toml  melee/combo_specials.toml          # 07 §4.4 tables 9-A, 9-B, 9-C, 9-D, 9-E, specials
  overrides/
    weapons/<slot>/<slug>.toml  attacks/<slug>.toml  enemies/<slug>.toml  co_names.toml  progenitor.toml
  aliases/
    inspiration.toml                   # inspiration names → our UniqueNames, slot ids 0-29, Incarnon {mId, pId}
    enemies.toml                       # wiki enemy renames
```

One file per item keeps reviews small and lets several curators work in parallel without merge conflicts (AD-12).
Owners (work-breakdown.md §8): `rules/` except `attack_roles` and `stat_lines` by T03b (`game_constants.toml`
included, DC-19), `rules/melee/` by T03c, `attack_roles.toml` and `overrides/{attacks,co_names}` by T18,
`stat_lines.toml` by T19, item directories, `overrides/weapons/<slot>/` (`<slot>` = the bundle weapon file name:
`primary` T20, `secondary` and `modular` T21, `melee` T22, `archgun`, `archmelee`, `companion` and `exalted` T29;
contract review N-9), `overrides/enemies/`, `overrides/progenitor.toml` and `aliases/enemies.toml` by the curation
tasks T20-T29 (the table of work-breakdown.md section 5), `aliases/inspiration.toml` by T10.
The authoring schema is the `EffectDef` of domain-model.md §11 plus the header of §15.1, and for the files that are not
items the authoring types of `wfd_model::curated` (contract review M-6, DC-18, DC-19): `WeaponOverrideFile`
(`overrides/weapons/**`), `AttackOverrideFile` (`overrides/attacks/*.toml`), `EnemyOverrideFile`
(`overrides/enemies/*.toml`), `ProgenitorFile` (`overrides/progenitor.toml`), `EnemyAliasFile` (`aliases/enemies.toml`),
`GameConstantsFile` (`rules/game_constants.toml`) and the rule-table types; `cargo xtask gen-schema` writes
`data/curated/schema.json` (schemars) for editor validation (Taplo), so curators can author these files before T18
reads them.

### 6.2 The three tiers

| Tier | Input | Covers | Output status |
|---|---|---|---|
| 1. Stat-line import (automatic) | `rules/stat_lines.toml`: the stat-line → bucket table of 04 §5.1 as anchored regex patterns over wiki `Description` lines and WFCD `levelStats` lines (`^\+(?P<v>[\d.]+)% Damage$` → `base_dmg`; `(x2 for Bows)` → `scope_factor`; `<DT_FIRE_COLOR>Heat` → `HEA`) | single-line unconditional effects; about 70 % of weapon mods (`DS:wfcd` §3.4) | `auto` when every line of the item maps; otherwise the item stays `unmodelled` and appears in `curate todo` |
| 2. Spec-table ingestion (automatic) | §5.2 | CO catalogue, status rules, faction modifiers, scaling | data on attacks and in `Rules` |
| 3. Curated (hand-written) | `data/curated/**/*.toml` | conditional, stacking and special effects, arcanes, shards, perks, sets, buffs, custom stats, riven mapping, rules, overrides | `verified`, `draft`, `unmodelled`, `no_damage_effect` |

An effect that uses a DSL construct without a spec vector yet (`count_within`, `counter_bonus.reset_on = heavy`,
outcome gates; `Per::Resolved` power curves and `Per::ReloadedFraction` except in the Plated Round form that TV-04-16
covers; overview R-2) cannot be `verified`: the validator caps the item at `draft` and the engine adds warning `effect.no_spec_vector`. Such items
are usable and are listed in the coverage report until the spec adds the vector.

A curated file for an item replaces its tier-1 effects entirely. Every item records which tier produced its effects;
the coverage report counts each tier. Tier-1 effects are emitted into `data/generated/items/*.json`, so they are
reviewed like any other data change.

### 6.3 Curator workflow

1. `wfd-datagen curate todo --kind weapon_mod` lists items whose status is `unmodelled` or `draft`, ordered by a
   priority list (`data/curated/priority.toml`: items in the inspiration's parity set and meta mods first), with
   their upstream text.
2. `wfd-datagen curate scaffold "Hunter Munitions"` writes `data/curated/weapon_mod/sets/hunter_munitions.toml`
   with the header, the upstream text of every rank as comments, `status = "draft"`, and draft effects for every
   line the tier-1 patterns map. Non-simple lines (`On Kill:`, `per Status Type`, `stacks with Combo Multiplier`,
   `(x2 for Heavy Attacks)`) become `# TODO` comments naming the DSL construct to use, never a guess.
3. The curator writes the effects from the spec and the cited wiki page, adds `sources`, sets
   `status = "verified"`.
4. `wfd-datagen curate rebind <file>` stores the upstream text and its SHA-256 and checks the bindings;
   `wfd-datagen check` runs all validators; `cargo xtask effect-smoke <slug>` evaluates the item on its reference
   weapon (testing.md §3) and proves that each effect reaches the stat it claims.
5. The pull request shows the TOML and the changed lines of `data/generated/`.

A new mechanic (a new `Target` variant, `Per`, `TriggerId` or mechanic kind) is an engine change plus a spec vector;
the validator rejects unknown names, so data can never reference behaviour that does not exist.

---

## 7. Validation and auto-patching

`wfd-datagen check` (CI) and every `build` run these checks. Errors stop the build; warnings go into `REPORT.md`,
`report.json` and the UI's data page. Thresholds and allow-lists live in `data/validation/`: `base.toml` (T18:
cross-source allow-lists, join thresholds) and `coverage/<task>.toml`, one file per curation task, so batches never
edit the same file.

| Check | Rule | Severity |
|---|---|---|
| Schema | TOML deserializes into the authoring types; unknown keys rejected; stat, scalar, trigger, mechanic and group names exist | error |
| References | every curated `id` exists in the joined catalogue (or is a `wfd:` item); `name` equals the catalogue English name; every override and CO row maps to a weapon/attack | error |
| Class | derived T-02 class (domain-model.md §12.1) equals the declared `class` when present | error |
| Element order | `order` legal for the element and item kind (domain-model.md §13) | error |
| Applicability | a stat that exists only for some weapon classes (combo, attack speed, multishot on melee) on an item that cannot equip such a weapon; scope roles that no compatible weapon has | error |
| Squad auras | a buff item (`ItemKind::Buff`, group `squad`) that encodes a squadmate's Corrosive Projection or Shield Disruption: those enter only through the T-23 options `squad_cp` and `squad_shield_disruption` (05 E-5.10, E-5.16), so a buff would count the aura twice (contract review M-19) | error |
| Binding | every `bind` refers to an existing number token; the bound value equals the token after unit conversion | error for `verified`, warning for `draft` |
| Unbound numbers | every number of the upstream max-rank text is bound or listed in `ignore_numbers` with a reason | error for `verified` |
| Rank scaling | for every rank in WFCD `levelStats`, the effect value at that rank, rendered by the canonical English renderer, matches the WFCD line within its 1-decimal rounding (`DS:wfcd` §3.4) | error for `verified` |
| Fingerprint | the upstream text SHA-256 equals the stored one; otherwise the item becomes `stale` (auto-patched when possible) | warning |
| Upgrade types | wiki `UpgradeTypes` map to expected targets (`WEAPON_DAMAGE_AMOUNT` → `base_dmg`, `WEAPON_FIRE_ITERATIONS` → `multishot`, `WEAPON_DAMAGE_IF_VICTIM_PROC_ACTIVE` → CO family); mismatch reported (the tags are sometimes wrong, `DS:wiki` §11) | warning |
| Ranges and units | fractions plausible per stat (declared in the stat registry), stacks ≥ 1, durations ≥ 0 (0 = no expiry, `StackModel.duration`; DC-21), Tauforged factor 1.5, chances in [0, 1] | error |
| Rule invariants | status caps > 0, every attack has ≥ 1 damage type and a fire model, every weapon has ≥ 1 attack, every faction has 18 modifiers, every melee weapon resolves a combo table for Neutral (E-7.0b) | error |
| Incarnon completeness | every curated Incarnon family has perks for tiers 2-5; families not yet curated are counted, not failed | warning; error only for a family whose file exists |
| Cross-source | wiki attack vs PE top-level stats per weapon (allow-list for summed attacks) | warning |
| Coverage | counts per status and kind against `data/validation/coverage/*.toml`: each file sets `min_modelled` per kind for its batch's directories (0 until the batch starts) and the batch raises it to its achieved count when it is accepted, so thresholds ratchet and never block a batch that has not run; a count below a stated minimum is an error. Targets (every item of the inspiration's parity set modelled, every arcane with a damage `UpgradeType`) are reported as progress, never as errors; items that cannot be encoded yet are listed in an `allow_unmodelled` table with a reason (an R-2 construct) | error below a stated minimum, else warning |
| Spec anchors | the inputs of spec 00 §3.8 (Braton Prime normal attack, Lancer, Serration, Split Chamber, Point Strike, Vital Sense, Infected Clip, Stormbringer, Primed Bane of Grineer, Speed Trigger) exist with exactly those values | error |

**Auto-patching number drift.** When the fingerprint of a curated item changes and the new upstream text has the same
token structure as the stored one (only numbers differ), the build rewrites the bound values in the generated data
(never in the curated source), sets the status `stale` with `auto_patched = true`, and lists the change in the
report. Example: Galvanized Aptitude "+40% Direct Damage per Status Type" becomes "+45% …"; the `co` effect bound to
`n2%` becomes 0.45 at once, the UI shows "auto-updated from game text, not yet reviewed", and a curator later
re-verifies and runs `curate rebind`. If the token structure changed, the item becomes `stale` without patching and
keeps its old numbers, flagged in the UI.

---

## 8. Output and bundle

### 8.1 Canonical JSON (`data/generated/`)

```text
data/generated/
  manifest.json                  Manifest (domain-model.md §8)
  weapons/{primary,secondary,melee,archgun,archmelee,exalted,companion,modular}.json
  items/{weapon_mod,weapon_arcane,frame_mod,aura,frame_arcane,shard,stance,incarnon,buff,custom}.json
  sets.json  stances.json  combo_tables.json  modular.json  frames.json  enemies/<module>.json  rules.json  names/en.json
                                 (enemies split by wiki sub-module of Enemies/data, §11: infestation, themurmur, …)
  stable_keys.json               append-only (§5.3)
  spec/{co_catalogue,co_defaults,status_summary,faction_modifiers,scaling_coefficients}.json   (also the embedded refresh inputs)
  REPORT.md  report.json         coverage, unmatched, conflicts, stale entries, warnings
  LICENSE                        CC BY-NC-SA 3.0 (AD-11)
```

Pretty-printed, keys sorted, two-space indent, floats with the shortest round-trip representation, records sorted by
`uniqueName`. A game patch therefore shows as a small, readable diff. Estimated size 8-12 MB (the wiki module JSON
alone is 6.4 MB for weapons, enemies, mods, arcanes, stances and modular parts, measured on
`research/external/cache/poc/out/`).

### 8.2 Binary bundle

```text
file   = magic "WFDPSB01" | u32 schema_version | u32 header_len | header | body
header = postcard(BundleHeader { schema_version, data_version, content_hash: [u8; 32] /* blake3(body) */,
                                 created_at, game_baseline, licences, counts })
body   = zstd frame of postcard(Bundle)
```

- `crates/wfd-data/build.rs` reads `WFD_DATA_DIR` (used by tests to point at `data/fixtures/mini/`), else
  `data/generated/` when it exists, else `data/fixtures/mini/` with a `cargo:warning` (overview.md AD-11; ruling
  T02c-R5: the crates keep compiling while `data/generated/` does not exist, and `xtask dist` refuses a fixture
  bundle), deserializes into `wfd-model` types (the schema check), runs the reference validator (stable keys unique,
  ids dense, effect targets exist), and writes `OUT_DIR/bundle.bin`, embedded with `include_bytes!`. A broken data file
  fails the build, not the user.
- Compression (contract review API-4, DC-28): `build.rs` compresses the embedded bundle at zstd level 19 (its
  build-dependency `zstd`); bundles written at run time (`wfd-datagen build --bundle`, the in-app refresh) use
  `wfd_data::encode_bundle`, which compresses with the pure-Rust encoder of `ruzstd` (its fastest level, the only
  compressing level ruzstd 0.9 implements), so no C library is linked into the shipped decode path. Readers decode
  both (the format is one zstd frame either way).
- Decoding at startup uses `ruzstd` (pure Rust) and `postcard`; target < 100 ms on one core, guarded by the
  `wfd-data` benchmark `bench_load`. `wfd_data::read_header` reads the header alone (magic and schema version
  checked, no decompression): the server lists user bundles and picks the newest loadable one with it (§9.2; API-2).
- Estimated size: 1-3 MB embedded (postcard is 3-4× smaller than JSON for numeric records, zstd another 3-5×;
  estimate, measured by task T18).
- Images are not bundled (§10).

### 8.3 Loading and indexes

```rust
// crates/wfd-data/src/lib.rs
pub struct GameData { pub bundle: Bundle, idx: Indexes }
impl GameData {
    pub fn embedded() -> Result<GameData, DataError>;
    pub fn from_bytes(bytes: &[u8]) -> Result<GameData, DataError>;          // user bundles
    pub fn from_json_dir(dir: &Path) -> Result<GameData, DataError>;          // tests, fixtures
    pub fn by_key(&self, k: StableKey) -> Option<Entity<'_>>;                 // Weapon | Item | Enemy
    pub fn by_unique_name(&self, n: &str) -> Option<Entity<'_>>;
    pub fn search(&self, q: &SearchQuery) -> Vec<SearchHit>;                  // localized substring + fuzzy, by kind/faction/stat
    pub fn candidates(&self, input: &CalcInput, slot: SlotId, query: Option<&str>, stat: Option<&str>)
        -> Vec<Candidate>;                                                     // legality per spec 09
    pub fn resolve(&self, input: &CalcInput) -> Result<(Resolved, Vec<Message>), Vec<Message>>;
                                              // violations inside Resolved; Ok carries the warnings (T02b-3)
    pub fn to_doc(&self, b: &Build, s: &Scenario) -> CalcInput;
    pub fn is_retired(&self, k: StableKey) -> bool;                           // Bundle.retired_keys (API-5)
}
```

`wfd-data::legality` is the single implementation of spec 09 (slot counts and kinds, compatibility, exclusion and
duplicates, ranks and drain, capacity and minimum Forma, arcane families, shards, Helminth, set counting, riven
legality, Incarnon, valence); the UI (through `/api/candidates` and `CalcReport.violations`), the build decoder and the
optimizer's pools all call it, and the optimizer computes capacity and riven generation with its functions
(`slot_cost`, `slot_charge`, `CapacityModel`, `riven_class`, `riven_traits_for`; contract review API-12, API-13).
Validation never blocks evaluation (09 §1.5). `resolve` returns the document issues it dropped with a warning beside
the resolved build (T02b-3); the server appends them to `CalcReport.warnings`.

---

## 9. Refresh after a game update

### 9.1 Developer path

```text
wfd-datagen fetch --update-lock     # new revids, new PE build label, newest WFCD tag
wfd-datagen build --offline         # regenerate data/generated/
wfd-datagen diff <git HEAD bundle> data/generated
```

Review `REPORT.md` (new items `unmodelled`, stale and auto-patched entries, conflicts), curate what changed, commit
`data/sources.lock.toml`, `data/generated/` and curated edits together.

### 9.2 User path (in-app)

| Step | Behaviour |
|---|---|
| Check | About → Data shows the active `data_version`, game baseline and sources. "Check for updates" (`POST /api/data/check`) asks the PE index (ETag) and the wiki revids (2-3 requests) whether anything changed |
| Refresh | `POST /api/data/refresh` runs `fetch` + `build` in a background job with SSE progress: about 60 wiki modules in 2 batched calls, ≤ 17 PE manifests, WFCD files for the newest tag (found through `registry.npmjs.org`); about 25 MB; 1-2 minutes at the polite rate. Inputs embedded in the binary (feature `refresh`, through the server's `build.rs` into `OUT_DIR`): `data/curated/`, the table list `data/spec_tables.toml` and the ingested spec tables `data/generated/spec/*.json` (`SpecSource::Ingested`; an empty placeholder while `data/spec_tables.toml` or `data/generated/spec/` is missing, before T18 writes them, so a refresh then fails with `DatagenError::Config` naming the missing file or spec tables; rulings T02c-R6, NEW-CR-1); the binary never reads `docs/`. No `data/validation/` is embedded: the checks run without allow-lists and coverage minimums, which guard curation, not user refreshes (`PipelineConfig::in_app`). A refresh stopped by validation ends with `failed` carrying its report (server-api.md §9; API-23). The CLI flag `--curated <dir>` lets a user point at a newer checkout |
| Keys | uniqueNames of the embedded bundle's live and retired key lists keep their global keys; new uniqueNames get session-local keys ≥ 2^31 (§5.3), stored in the user bundle; share codes and persisted lists that contain a local key carry its uniqueName (AD-18), so they decode on any machine and after any upgrade (by uniqueName; an unknown uniqueName is reported as `DocIssue::ItemUnknown`) |
| Report | field diffs against the active bundle, new items (`unmodelled` until curated), stale and auto-patched curated entries, conflicts |
| Activate | writes `<data_dir>/bundles/wfdata-<build_label>-<hash8>.bin` and swaps the server's `ArcSwap<GameData>`; in-flight requests and optimizer jobs finish on the old snapshot |
| Rollback | the previous bundles stay; "Activate" on an older entry, or the embedded bundle, switches back |
| Startup | the newest user bundle whose `schema_version` equals the binary's is loaded (headers read with `wfd_data::read_header`, no body decoded); otherwise the embedded one |

Builds that refer to removed items decode with a warning per missing item (`DocIssue::ItemRetired`). The CLI
equivalent is `warframe-dps data status | check | refresh [--curated DIR] | list | activate <id>`.

---

## 10. Licences and attribution

AD-11 (overview.md): publication of the wiki-derived data is the user's decision (Q-U1, pending). Until it is
made, `data/generated/` is committed to the private repository with the notices below, and nothing is published.

| Item | Notice |
|---|---|
| `data/generated/LICENSE` | CC BY-NC-SA 3.0 full reference (`https://creativecommons.org/licenses/by-nc-sa/3.0/`) for the derived data |
| `data/NOTICE.md` | wiki: "Contains data from wiki.warframe.com Lua modules (list with revids), CC BY-NC-SA 3.0; modified: converted, joined, normalized"; WFCD: MIT copyright line "Copyright (c) 2017 Kaptard"; Public Export: "Warframe content and data © Digital Extremes Ltd.; used for identifiers; this project is not affiliated with Digital Extremes"; curated tables: project licence |
| Bundle | `Manifest.licences: Vec<LicenceNotice { source, licence, url, attribution, modules: Vec<(title, revid)> }>` generated from the same records |
| UI | About page renders the licences from `/api/meta`; footer link "Data: CC BY-NC-SA 3.0 (wiki.warframe.com)" |
| Images | never bundled; `/api/img/{name}` proxies `content.warframe.com/PublicExport/<textureLocation>` (§3.2) on first use into `<data_dir>/images/` (switchable off with `images.fetch = false`); offline, and in every test and visual baseline (`images.fetch = false`), the UI shows generated glyph tiles |
| Not committed | `.cache/`, `research/external/`, raw upstream files |

---

## 11. Module and file inventory

| Source | Items fetched |
|---|---|
| wiki modules | `Weapons/data/{primary,secondary,melee,archwing,companion,modular,misc}` (railjack excluded), `Enemies/data/{grineer,corpus,infestation,orokin,sentient,stalker,narmer,themurmur,techrot,scaldra,anarchs,unaffiliated}`, `DamageTypes/data`, `Stances/data`, `Modular/data`, `Modular/data/kitgun` (cross-check only), `Mods/data`, `Mods/itemCompatibility/data`, `Arcane/data`, `Warframes/data`, `Upgrades/data`, `Enum/data`. Not used: `Weapons/ppdata`, `Enemies/data/pre-U36`, `Flaicher/dispoData`, `Maximization/data`, `Ability/data` (stale stats, `F:buffs` §13) |
| Public Export | `index_en.txt.lzma`, `ExportWeapons_en.json`, `ExportUpgrades_en.json` (mods, mod sets, riven `upgradeEntries`), `ExportRelicArcane_en.json` (arcanes), `ExportWarframes_en.json` (warframe uniqueNames and polarities; `ExportAbilities` Helminth abilities, for buff ids), `ExportSentinels_en.json`, `ExportManifest.json` (image `textureLocation`s) |
| WFCD | `Primary.json`, `Secondary.json`, `Melee.json`, `Arch-Gun.json`, `Arch-Melee.json`, `SentinelWeapons.json`, `Warframes.json` (names), `Mods.json`, `Arcanes.json`, `Misc.json` (exalted weapons, kitgun parts); later `i18n/<locale>.json`; version discovery `registry.npmjs.org/@wfcd/items` |
