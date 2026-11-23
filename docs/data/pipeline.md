Status: draft (architecture)

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
| wiki.warframe.com Lua data modules (`DS:wiki`) | weapon attacks (per-attack damage map per pellet, crit, status, fire rate, multishot, falloff, radius, forced procs, charge, burst, spool, ammo cost, Incarnon form attacks, `AttackParentIndex`), melee fields, enemies (1,019: faction, base level, pools, body-part prose, `ProcResists`, Eximus fields), `DamageTypes/data` faction table, `Stances/data` combos, `Modular/data` kitgun/zaw parts, `Mods/data` and `Arcane/data` identity (description text, `Incompatible`, `Type`, `UpgradeTypes`, set membership), `Mods/itemCompatibility/data` | effect values (prose only) | CC BY-NC-SA 3.0 (`meta=siteinfo&siprop=rightsinfo`) |
| DE Public Export (`DS:pe`) | canonical `uniqueName` of every DE item, `buildLabel` freshness, top-level weapon stats (cross-check only), trigger, mastery, polarities, mod drain/rarity/max rank/`compatName`, `ExportModSet`, arcane catalogue, riven `upgradeEntries` base values per class | attacks, enemies, effects (display text only) | none stated (`DS:pe` open question 1): used for identifiers and numbers that WFCD (MIT) already redistributes |
| WFCD warframe-items, pinned tag (`DS:wfcd`) | English names, descriptions, image names, other locales later, per-rank `levelStats` text (rank checks of curated effects), catalogue cross-check | attacks (lossy), enemies (stale damage model), effects | MIT |
| Spec tables (`docs/mechanics/`) | tables the spec marks as normative or ingestable: CO catalogue (04 §2.14), status summary (03 §4.1), faction modifiers (05 §5.1), scaling coefficients (05 §6.1) | — | project |
| Curated tables (`data/curated/`) | every effect that is not a simple stat line, rules not in the spec tables, overrides (06 §4.3, 05 §4.5), aliases | — | project |

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
pub struct PipelineConfig { pub cache_dir: PathBuf, pub curated: CuratedSource /* Dir(PathBuf) | Embedded */,
                            pub spec_dir: Option<PathBuf>, pub offline: bool, pub lock: LockMode /* Pinned | Latest */ }
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
| Hosts allowed | `wiki.warframe.com`, `content.warframe.com`, `raw.githubusercontent.com` (WFCD), images: the WFCD CDN host; nothing else |
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
Arch-Gun, Arch-Melee, SentinelWeapons, Mods, Arcanes, Misc); each verified by SHA-256 from `sources.lock.toml`.
`--update-lock` moves to the newest tag.

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

---

## 4. Parsing

### 4.1 Wiki Lua modules (mlua)

Port of the proof of concept in `research/external/cache/poc/lua2json/` (`DS:wiki` §9):

- `mlua` 0.12 with features `lua51`, `vendored`, `send`. Only the `math`, `string` and `table` libraries are loaded;
  `io`, `os`, `package`, `debug`, `load`, `dofile`, `require` are absent.
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
| top-level weapon crit, status, fire rate, magazine, reload | wiki (per attack) | PE | relative difference > 1e-4 reported; PE sums attacks for some weapons (Kuva Bramma), allow-list in `data/validation.toml` |
| attacks, forced procs, falloff, charge, burst, spool, Incarnon form attacks | wiki | — | — |
| names, descriptions, images, locales | WFCD | wiki `Name` | — |
| enemies (all fields) | wiki | — | WFCD enemies never used (`DS:wfcd` §6) |
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
columns = ["#", "Class", "Weapon(s)", "Attack", "Behaviour", "B listed", "B_co wiki", "ρ_co", "co_ref", "B_co^abs", "Flags"]

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
("Braton/Mk1/Prime/Vandal") to every listed weapon, map wiki attack names to bundle attacks, and become `CoSpec` on
the attack; rows that do not map are listed in the report and fixed in `data/curated/overrides/co_names.toml`
(names only, never numbers). Attacks without a row get the default rule of 04 §2.5. The generated form is written to
`data/generated/spec/*.json` for review; the spec stays the only place these numbers are written. The bundle manifest
records the hash of the ingested tables (`spec_rev`).

### 5.3 Derive

| Step | Rule |
|---|---|
| Attack key | `AttackKey { slug: slugify(AttackName), idx }`; duplicate slugs get `_2`, `_3` |
| Attack roles | from `AttackName`, `ShotType`, `AttackParentIndex`, Incarnon markers and melee fields, by the rule table `data/curated/rules/attack_roles.toml` (regex on attack names → roles) |
| Parent links and groups | `AttackParentIndex` with the override table `data/curated/overrides/attacks/*.toml` for the inconsistent cases (`DS:wiki` §11; 06 §9.1); `comb` groups for T-22 from children of one parent plus `overrides` |
| Trigger | normalisation of the 33 wiki strings (06 §4.2) and per-attack overrides |
| Hand tables of 06 §4.3 | spool cadence, Kohm family, beam ramp start, per-shell reload, semi-cap overrides, bow doubling, fixed charge time, no-reload weapons, exalted ammo mode, reload-triggered attacks, whole-magazine attacks, sniper combo exclusions: `data/curated/overrides/weapons/<slug>.toml` |
| Enemy body parts | parse `Multis` prose into `BodyPart`s; head-crit capability by the rules of 05 §4.3; overrides in `data/curated/overrides/enemies/*.toml` |
| Enemy special DR | 05 §4.5 table in `data/curated/rules/special_dr.toml` |
| Compatibility | wiki `Type` → categories via `Mods/itemCompatibility/data`; compat tags; `uncomptag`; Semi-only trigger requirement |
| Modular parts | kitgun chambers/grips/loaders, zaw strikes/grips/links from `Modular/data` (assembled at runtime, `wfd-engine::assemble`) |
| Stable keys | `data/generated/stable_keys.json` is append-only: new uniqueNames get the next integer; removed ones keep their key and are listed as `retired`; a key is never reused |

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
  incarnon/<family>.toml               # one file per Incarnon family (tiers, perks, variant values)
  buff/<group>/<slug>.toml             # ability, focus, companion, squad, debuff, environment
  custom/<slug>.toml                   # the 26 custom stats of spec 00 §6.3
  rules/
    mult_groups.toml  slot_layout.toml  riven_traits.toml  riven_formula.toml  body_parts.toml
    special_dr.toml  toggles.toml  attack_roles.toml  stat_lines.toml  factions.toml  categories.toml
  overrides/
    weapons/<slug>.toml  attacks/<slug>.toml  enemies/<slug>.toml  co_names.toml  progenitor.toml
  aliases/
    inspiration.toml                   # inspiration names → our UniqueNames, slot ids 0-29, Incarnon {mId, pId}
    enemies.toml                       # wiki enemy renames
```

One file per item keeps reviews small and lets several curators work in parallel without merge conflicts (AD-12).
The authoring schema is the `EffectDef` of domain-model.md §11 plus the header of §15.1; `cargo xtask gen-schema`
writes `data/curated/schema.json` (schemars) for editor validation (Taplo).

### 6.2 The three tiers

| Tier | Input | Covers | Output status |
|---|---|---|---|
| 1. Stat-line import (automatic) | `rules/stat_lines.toml`: the stat-line → bucket table of 04 §5.1 as anchored regex patterns over wiki `Description` lines and WFCD `levelStats` lines (`^\+(?P<v>[\d.]+)% Damage$` → `base_dmg`; `(x2 for Bows)` → `scope_factor`; `<DT_FIRE_COLOR>Heat` → `HEA`) | single-line unconditional effects; about 70 % of weapon mods (`DS:wfcd` §3.4) | `auto` when every line of the item maps; otherwise the item stays `unmodelled` and appears in `curate todo` |
| 2. Spec-table ingestion (automatic) | §5.2 | CO catalogue, status rules, faction modifiers, scaling | data on attacks and in `Rules` |
| 3. Curated (hand-written) | `data/curated/**/*.toml` | conditional, stacking and special effects, arcanes, shards, perks, sets, buffs, custom stats, riven mapping, rules, overrides | `verified`, `draft`, `unmodelled`, `no_damage_effect` |

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
   `wfd-datagen check` runs all validators; `cargo test -p wfd-engine --test effect_smoke -- <slug>` evaluates the item
   on its reference weapon (testing.md §3) and proves that each effect reaches the stat it claims.
5. The pull request shows the TOML and the changed lines of `data/generated/`.

A new mechanic (a new `Target` variant, `Per`, `TriggerId` or mechanic kind) is an engine change plus a spec vector;
the validator rejects unknown names, so data can never reference behaviour that does not exist.

---

## 7. Validation and auto-patching

`wfd-datagen check` (CI) and every `build` run these checks. Errors stop the build; warnings go into `REPORT.md`,
`report.json` and the UI's data page. Thresholds and allow-lists are in `data/validation.toml`.

| Check | Rule | Severity |
|---|---|---|
| Schema | TOML deserializes into the authoring types; unknown keys rejected; stat, scalar, trigger, mechanic and group names exist | error |
| References | every curated `id` exists in the joined catalogue (or is a `wfd:` item); `name` equals the catalogue English name; every override and CO row maps to a weapon/attack | error |
| Class | derived T-02 class (domain-model.md §12.1) equals the declared `class` when present | error |
| Element order | `order` legal for the element and item kind (domain-model.md §13) | error |
| Applicability | a stat that exists only for some weapon classes (combo, attack speed, multishot on melee) on an item that cannot equip such a weapon; scope roles that no compatible weapon has | error |
| Binding | every `bind` refers to an existing number token; the bound value equals the token after unit conversion | error for `verified`, warning for `draft` |
| Unbound numbers | every number of the upstream max-rank text is bound or listed in `ignore_numbers` with a reason | error for `verified` |
| Rank scaling | for every rank in WFCD `levelStats`, the effect value at that rank, rendered by the canonical English renderer, matches the WFCD line within its 1-decimal rounding (`DS:wfcd` §3.4) | error for `verified` |
| Fingerprint | the upstream text SHA-256 equals the stored one; otherwise the item becomes `stale` (auto-patched when possible) | warning |
| Upgrade types | wiki `UpgradeTypes` map to expected targets (`WEAPON_DAMAGE_AMOUNT` → `base_dmg`, `WEAPON_FIRE_ITERATIONS` → `multishot`, `WEAPON_DAMAGE_IF_VICTIM_PROC_ACTIVE` → CO family); mismatch reported (the tags are sometimes wrong, `DS:wiki` §11) | warning |
| Ranges and units | fractions plausible per stat (declared in the stat registry), stacks ≥ 1, durations > 0, Tauforged factor 1.5, chances in [0, 1] | error |
| Rule invariants | status caps > 0, every attack has ≥ 1 damage type and a fire model, every weapon has ≥ 1 attack, every Incarnon family has perks for tiers 2-5, every faction has 18 modifiers | error |
| Cross-source | wiki attack vs PE top-level stats per weapon (allow-list for summed attacks) | warning |
| Coverage | counts per status and kind; minimums from `data/validation.toml` (every item of the inspiration's parity set modelled; every arcane with a damage `UpgradeType`); regressions against the committed `report.json` | error below thresholds, else warning |
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
  sets.json  stances.json  modular.json  enemies/{<faction>}.json  rules.json  names/en.json
  stable_keys.json               append-only (§5.3)
  spec/{co_catalogue,status_summary,faction_modifiers,scaling_coefficients}.json
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
body   = zstd(level 19, postcard(Bundle))
```

- `crates/wfd-data/build.rs` reads `data/generated/` (or `WFD_DATA_DIR`, used by tests to point at
  `data/fixtures/mini/`), deserializes into `wfd-model` types (the schema check), runs the reference validator
  (stable keys unique, ids dense, effect targets exist), and writes `OUT_DIR/bundle.bin`, embedded with
  `include_bytes!`. A broken data file fails the build, not the user.
- Decoding at startup uses `ruzstd` (pure Rust) and `postcard`; target < 100 ms on one core, guarded by the
  `wfd-data` benchmark `bench_load`.
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
    pub fn candidates(&self, b: &Build, s: &Scenario, slot: SlotId) -> Vec<Candidate>;   // legality per spec 09
    pub fn resolve(&self, input: &CalcInput) -> Result<Resolved, Vec<Message>>;   // violations inside Resolved
    pub fn to_doc(&self, b: &Build, s: &Scenario) -> CalcInput;
}
```

`wfd-data::legality` is the single implementation of spec 09 (slot counts and kinds, compatibility, exclusion and
duplicates, ranks and drain, capacity and minimum Forma, arcane families, shards, Helminth, set counting, riven
legality, Incarnon, valence); the UI (through `/api/candidates` and `CalcReport.violations`), the build decoder and the
optimizer's pools all call it. Validation never blocks evaluation (09 §1.5).

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
| Refresh | `POST /api/data/refresh` runs `fetch` + `build` in a background job with SSE progress: about 60 wiki modules in 2 batched calls, ≤ 17 PE manifests, WFCD files for the newest tag; about 25 MB; 1-2 minutes at the polite rate. The curated tables used are those embedded in the binary (`include_dir!` of `data/curated/` at compile time); the CLI flag `--curated <dir>` lets a user point at a newer checkout |
| Report | field diffs against the active bundle, new items (`unmodelled` until curated), stale and auto-patched curated entries, conflicts |
| Activate | writes `<data_dir>/bundles/wfdata-<build_label>-<hash8>.bin` and swaps the server's `ArcSwap<GameData>`; in-flight requests and optimizer jobs finish on the old snapshot |
| Rollback | the previous bundles stay; "Activate" on an older entry, or the embedded bundle, switches back |
| Startup | the newest user bundle whose `schema_version` equals the binary's is loaded; otherwise the embedded one |

Builds that refer to removed items decode with a warning per missing item (`DocIssue::ItemRetired`). The CLI
equivalent is `warframe-dps data status | check | refresh [--curated DIR] | list | activate <id>`.

---

## 10. Licences and attribution

Decision AD-11 (overview.md): the wiki-derived `data/generated/` is committed.

| Item | Notice |
|---|---|
| `data/generated/LICENSE` | CC BY-NC-SA 3.0 full reference (`https://creativecommons.org/licenses/by-nc-sa/3.0/`) for the derived data |
| `data/NOTICE.md` | wiki: "Contains data from wiki.warframe.com Lua modules (list with revids), CC BY-NC-SA 3.0; modified: converted, joined, normalized"; WFCD: MIT copyright line "Copyright (c) 2017 Kaptard"; Public Export: "Warframe content and data © Digital Extremes Ltd.; used for identifiers; this project is not affiliated with Digital Extremes"; curated tables: project licence |
| Bundle | `Manifest.licences: Vec<LicenceNotice { source, licence, url, attribution, modules: Vec<(title, revid)> }>` generated from the same records |
| UI | About page renders the licences from `/api/meta`; footer link "Data: CC BY-NC-SA 3.0 (wiki.warframe.com)" |
| Images | never bundled; `/api/img/{name}` proxies the WFCD image host on first use into `<data_dir>/images/` (switchable off with `images.fetch = false`); offline, the UI shows generated glyph tiles |
| Not committed | `.cache/`, `research/external/`, raw upstream files |

---

## 11. Module and file inventory

| Source | Items fetched |
|---|---|
| wiki modules | `Weapons/data/{primary,secondary,melee,archwing,companion,modular,misc}` (railjack excluded), `Enemies/data/{grineer,corpus,infestation,orokin,sentient,stalker,narmer,themurmur,techrot,scaldra,anarchs,unaffiliated}`, `DamageTypes/data`, `Stances/data`, `Modular/data`, `Modular/data/kitgun` (cross-check only), `Mods/data`, `Mods/itemCompatibility/data`, `Arcane/data`, `Upgrades/data`, `Enum/data`. Not used: `Weapons/ppdata`, `Enemies/data/pre-U36`, `Flaicher/dispoData`, `Maximization/data`, `Ability/data` (stale stats, `F:buffs` §13) |
| Public Export | `index_en.txt.lzma`, `ExportWeapons_en.json`, `ExportUpgrades_en.json` (mods, mod sets, riven `upgradeEntries`), `ExportRelicArcane_en.json` (arcanes), `ExportWarframes_en.json` (Helminth abilities, for buff ids), `ExportSentinels_en.json`, `ExportManifest.json` (image paths, optional) |
| WFCD | `Primary.json`, `Secondary.json`, `Melee.json`, `Arch-Gun.json`, `Arch-Melee.json`, `SentinelWeapons.json`, `Mods.json`, `Arcanes.json`, `Misc.json` (exalted weapons, kitgun parts); later `i18n/<locale>.json` |
