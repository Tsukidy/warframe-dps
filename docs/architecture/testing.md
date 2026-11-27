Status: reviewed (architecture)

# Testing

The spec is the source of expected numbers; in-game measurements beat everything (spec 00 §1.4); the inspiration is
a test oracle for parity, never for game truth. This document lists the test layers, their tools, where the tests
live, and the CI gates. Every task of `work-breakdown.md` names the commands of this document as its acceptance
criteria.

Contents: 1 Layers · 2 Engine · 3 Effects and curated data · 4 Data pipeline · 5 Oracle golden tests · 6 Optimizer ·
7 Server · 8 Frontend · 9 Performance · 10 CI gates and commands · 11 Test data locations

---

## 1. Layers

| Layer | What | Tools | Where |
|---|---|---|---|
| Spec vectors | every `TV-nn-mm` of `docs/mechanics/*.md` (01-09, 156 vectors today) and the spec 00 §3.8 worked example | custom harness, probes | `crates/wfd-engine/tests/spec_vectors.rs`, `spec_conformance.rs`, `tests/spec_vectors/*.toml`; build-rule vectors (09) in `crates/wfd-data/tests/spec_vectors_09.rs`; pipeline-side melee vectors (TV-07-10, TV-07-12) in `crates/wfd-datagen/tests/spec_vectors_07.rs` |
| Engine units | one test per formula or decision without a numeric vector (E-11 cases, CD-04 ordering, D-0xx decisions, NP-1/NP-2 f32 points) | `cargo test` | `crates/wfd-engine/src/**` `mod tests` |
| Coverage matrix | one fixture item per row of domain-model.md §17 with a hand-computed expected value | `cargo test` | `crates/wfd-engine/tests/effects_coverage.rs` |
| Mechanics | each mechanic kind in EV and MC on a minimal scenario | `cargo test` | `crates/wfd-engine/tests/mechanics.rs` |
| Properties | slot-permutation bit identity, incremental = full compile, monotonicity of `Inc` slots, `Bound ≥ WindowExact` (score keys, every metric), `WindowFast` error ≤ ε, quantization sum bounds, MC thread independence | proptest 1.12 (fixed seeds in CI, 10k cases nightly) | `crates/wfd-engine/tests/properties.rs` |
| Reader fixtures | one fixture per non-monotone reader kind (§2.3) proving the slot downgrade | `cargo test` | `crates/wfd-engine/tests/monotone_readers.rs` |
| EV vs MC | 08 E-8.24 on the golden scenarios of 08 §12 | 1,000 runs in PR CI, 10,000 nightly | `crates/wfd-engine/tests/ev_vs_mc.rs` |
| Compat flags | each flag changes only its own register row's output | `cargo test` | `crates/wfd-engine/tests/compat.rs` |
| Effect smoke | every curated item reaches its declared stats, only inside its scope | `cargo xtask effect-smoke` (needs `wfd-data` and the generated bundle) | `xtask/src/effect_smoke.rs` (T19) |
| Data pipeline | parsing, sandbox, join, validators, bundle, diff | `cargo test`, insta 1.49 | `crates/wfd-datagen/tests/`, `crates/wfd-data/tests/` |
| Oracle golden | inspiration run headless, seeded, compared with our compat and game profiles | Node + Playwright (recording), Rust (comparison) | `tools/oracle/`, `crates/wfd-engine/tests/golden_oracle.rs` |
| In-game measurements | `research/test-cases/ingame/*.toml` | `cargo test` | `crates/wfd-engine/tests/measured.rs` |
| Optimizer | exactness, bounds, determinism, constraints, cancellation | `cargo test`, proptest | `crates/wfd-opt/tests/`; "beats WFOpt" in `cargo xtask oracle-audit` (§6) |
| Build codes | round trips, version samples, inspiration import | `cargo test`, proptest | `crates/wfd-buildcode/tests/` |
| Server | routes, SSE, security, CLI | `tokio::test`, `tower::ServiceExt::oneshot`, insta | `crates/wfd-server/tests/` |
| Frontend units | store, history, URL sync, formatters, schema-driven renderers, token contrast | Vitest 5.1 + Testing Library | `frontend/src/**/*.test.ts(x)` |
| End-to-end | one spec per parity row, keyboard flows, share/import, optimizer run and cancel | Playwright 1.65 (Chromium) | `frontend/e2e/` |
| Accessibility | axe on every page and open dialog | @axe-core/playwright 4.14 | every e2e test + `frontend/e2e/a11y.spec.ts` |
| Visual | screenshots of fixed scenarios, dark and light, 375 / 900 / 1440 px | Playwright `toHaveScreenshot` | `frontend/e2e/visual.spec.ts` |
| Performance | criterion benches, optimizer benchmark with node budget | criterion 0.8, `cargo xtask bench-check` | `crates/*/benches/` |
| Lints | fmt, clippy `-D warnings`, `xtask lint-engine`, `xtask spec-sync`, ESLint (typescript-eslint 8.74 with TypeScript 6.0.3) with the i18n rule, `tsc --noEmit` | — | CI |

---

## 2. Engine

### 2.1 Spec vectors

Engine.md §14 defines the twin files, the transcription check and the execution harness. Rules:

- Every `TV-nn-mm` id appearing in `docs/mechanics/nn-*.md` has an entry in `tests/spec_vectors/nn.toml`, and every
  entry's `source` points at a heading of its document. Entries whose source is a worked example rather than a TV id
  (spec 00 §3.8 → `TV-00-01`) are allowed; their expected numbers must still occur verbatim in that section.
- Every expected number in a twin occurs verbatim (as written, after stripping thousands separators) in the Markdown
  of its vector section. A typo fails `spec_conformance`.
- Tolerances are the document's own (02 §9: f32 values bit-exact, f64 within 1e-9 relative; 01 §8: 1e-9 relative;
  worked examples with rounded display values: relative tolerance equal to the printed precision).
- A vector whose engine part does not exist yet is marked `pending = "T14"` (the task that will make it pass); CI
  counts pending vectors and fails when a task's acceptance says they must pass.
- `cargo xtask spec-vectors --check` fails when a fenced `testvector` block in a topic document differs from its
  extracted twin.

### 2.2 Units, coverage, mechanics, compat

- Stage functions have unit tests against their E-number, including `f32` emulation (NP-1: Q = 2.000488519668579 for
  CDM0 = 2.0; NP-2: Grineer health 294.4855 at level 100) and quantization edge cases (`|r|·32` exactly on a half
  step, negative ratios, TV-01-17).
- Coverage: one fixture per domain-model.md §17 row, expected values computed by hand with `python3` and written in
  the test with a comment showing the arithmetic.
- Mechanics: each kind of domain-model.md §11.2 evaluated in EV and MC on a minimal scenario; EV equals the MC mean
  within tolerance.
- Compat: for each `CompatFlags` flag, a targeted fixture where only that register row's output changes; the test
  asserts that switching any other flag does not change it.

### 2.3 Properties (proptest)

| Property | Statement |
|---|---|
| Permutation | permuting non-element slots of a build gives bit-identical `CalcReport` numbers (`f64::to_bits`) (I-4) |
| Incremental | `compile_build(b) == compile_build(b − x) + compile_pick(x)` bit for bit in the `StatVec` |
| Monotonicity | increasing any slot that is `Inc` in the compiled scenario (`CompiledScenario.slot_class`) never lowers `Steady`, `WindowFast` or `WindowExact` scores (failing seeds become fixtures) |
| Reader downgrades | random generation alone would rarely hit the non-monotone readers, so each has a fixed fixture (`tests/monotone_readers.rs`): Devouring Attrition, Overwhelming Attrition, Melee Duplicate (`OnTier1Crit`), Secondary Enervate, Prelude of Might, a `distinct_below` predicate, Primed Chamber, Charged Chamber, Synth Charge, Plated Round. Each fixture shows a counterexample to the declared class (a larger slot value lowers the score) and asserts that the scenario compiler marks the fed slots `Unknown`, and that the slots stay `Inc` in the branch without the reader |
| Bound validity | for random partial builds and completions, `Kernel::bound(relaxed) ≥ score(completion, WindowExact)` in the `Score` order, for every metric including TTK across bands |
| Fast error | `|WindowFast − WindowExact| / WindowExact ≤ ε` per weapon class (ε = 0.5 % unless the class is listed over budget) |
| Quantization | Σ q_t is within `n/64` of Σ r_t for n nonzero types |
| Determinism | `calculate` twice, and MC with rayon pools of 1 and 8 threads, give byte-identical serialized reports (I-5) |
| Cross-platform | the CI matrix (Linux, macOS, Windows) produces equal hashes of the reports of 10 fixed scenarios |

### 2.4 EV vs MC

08 E-8.24: `|dps_window − x̄| ≤ 2.5758293035489004·SE + 0.02·|x̄|` with n_runs ≥ 1000, on the golden scenarios of
08 §12 (TV-08-03, TV-08-05, TV-08-06 and every `research/test-cases/` scenario with an EV and an MC expectation) and
the fixture corpus (§11). Scenarios in a gap class G-1 to G-11 of 08 §12 assert the stated sign and size of the gap
instead. The file is owned by T17 (it needs both evaluators).

### 2.5 Lints

- `cargo xtask lint-engine`: no float literals outside `spec_constants.rs` and tests in `wfd-engine` and `wfd-opt`
  (allow-list 0, 1, 2, 0.5) (I-1); no `/Lotus/` anywhere in their sources, comments included, and no item name of
  the mini fixture outside comments, where a whole identifier that is model vocabulary (a generated `CompatFlags`
  name, a stat-registry row or variant, a resolved-stat name) does not count and custom-stat records are not items
  (I-2, CONTRACTS.md C-22); `clippy::disallowed_types` forbids `HashMap`/`HashSet` in result paths.
- `cargo xtask spec-sync`: `CompatFlags` ↔ every register row whose switch kind is `Compat` (spec 00 §5.1 rule 3),
  over spec 00 §5.2 and the "Inspiration differences" register of every topic document 01-09 (the count follows the
  registers: 74 flags when T01d landed; the `xtask` tests pin it with the generated file); `TOGGLE_MAP` ↔ spec 00
  §6.1 (`Proposed` rows allowed); every `TV-nn-mm` of 00-09 present in the twin files (with `spec-vectors`).

---

## 3. Effects and curated data

**Effect smoke test** (`cargo xtask effect-smoke [<slug>…]`, `xtask/src/effect_smoke.rs`, owned by T19): it depends
on `wfd-data` and `wfd-engine` and loads the generated bundle (`data/generated/`), so `wfd-engine`'s own tests keep
needing only the mini fixture (overview §3.1). For every curated item, it picks the reference scenario (a weapon of the
item's class from `data/fixtures/reference_weapons.toml` resolved in the real bundle, a fixed enemy), evaluates with
and without the item (and with each conditional effect forced on), and asserts that every declared target changes in
the declared direction on the attacks and hits in its scope and does not change outside the scope. This catches
mis-scoped effects that would silently do nothing. Curation tasks run it for their slugs; the full run is a CI gate.
It also asserts that every derived MC mechanic id of the real bundle hashes to a distinct FNV-1a value (engine.md
§8.2).

**Validators** (pipeline.md §7): each check has one passing and one failing fixture under
`crates/wfd-datagen/tests/fixtures/curated/`. `wfd-datagen check` on the committed `data/curated/` is a CI gate.

---

## 4. Data pipeline

- **No network in unit tests.** Wire parsing runs on trimmed upstream excerpts committed under
  `crates/wfd-datagen/tests/fixtures/upstream/` (small, with licence notes); the fetcher is tested against a local
  HTTP mock (ETag, revid, 429 with `Retry-After`, 404 caching, cancellation, `--from-lock` with a revid and a hash).
- **Full-data tests** (`#[ignore]`, run with `-- --ignored full_cache`) read the cache that
  `wfd-datagen fetch --from-lock` restores (pipeline.md §3.3), never `research/external/cache/`, so a fresh worktree can
  run them after one fetch.
- **Lua sandbox**: no `io`/`os`/`package`/`load`; instruction and memory limits trip on hostile fixture modules;
  every cached module converts (an ignored test runs over the full cache locally).
- **Typos and unknown fields**: alias fixtures; unknown fields counted in the report.
- **Join thresholds**: WFCD→PE 668/668 weapons; wiki weapons→PE ≥ 825/872; wiki mods→PE ≥ 1,451/1,635; warframes wiki→PE: every
  record of the 126 frames (archwings, necramechs and operators are excluded) (full-data tests, run by T18 and in the
  data-update workflow).
- **Reproducible build**: `build --offline` twice, then `git diff --exit-code data/generated` (exit code 1 on any
  difference).
- **Modular assembly**: `assemble` equals every pre-expanded wiki kitgun combo (1,200) on damage and fire rate.
- **Spec tables**: each ingested table has a golden JSON snapshot (insta); a changed header fails.
- **Anchors**: the spec 00 §3.8 inputs are present with exactly those values.
- **Bundle**: JSON → bundle → `GameData` round trip preserves counts and anchors; `schema_version` mismatch is rejected;
  `bench_load` < 100 ms.
- **Diff**: `wfd-datagen diff` output snapshot on two fixture versions.
- **Stable keys**: a removed item keeps its key as retired; keys are never reused (property test over simulated
  updates).

---

## 5. Oracle golden tests

### 5.1 Harness

`tools/oracle/` (Node 22 + Playwright with Chromium, per `research/findings/oracle-feasibility.md`):

- Loads the local inspiration copy from `research/external/inspiration/` (gitignored), intercepting `/`, `/min.js`,
  `/jquery-min.js`, `/style-min.css`, `/data/json/enemies-min.json`, `/getWeapon.php?name=X` (local samples; missing
  weapons are fetched once from the live site politely and cached), `/getImg.php` (1×1 data URI); every other host
  returns 204.
- Seeds `Math.random` with a seeded generator (`page.addInitScript`, mulberry32 keyed by the run seed) so each run is
  reproducible.
- Opens the URL produced by `POST /api/build/export-inspiration` for a scenario (or a URL from
  `research/test-cases/*.json`), waits for the recalculation, and reads the result panels and the weapon and enemy
  panels from the DOM (layer × attack: random queue DPS, avg per bullet direct/status/total, median per bullet, TTK;
  per-type modded damage; scaled enemy values). For the optimizer it calls `WFOpt.runFull` and records the best build.
- `cargo xtask oracle-record [--cases research/test-cases/*.json tools/oracle/scenarios/*.json] [--runs 200]` runs
  R ≥ 200 seeds per scenario (08 §11.2, mulberry32 seeded with r) and writes `crates/wfd-engine/tests/golden/oracle/<case>.json` with the numbers only, the
  inspiration bundle SHA-256, the date and the seeds. Recording is a manual step because the inspiration copy is not
  in the repository; the recordings are committed and the comparison runs in normal CI.

### 5.2 Comparison classes

The comparison is spec 08 E-8.25, reproduced here (08 is normative):

```text
E-8.25 Oracle comparisons (compat profile unless stated)
  C1 deterministic panels: weapon per-type damage, CC, CDM, SC, MS, fire rate, enemy pools, armor, DR must equal the
     inspiration's displayed value after rounding ours to the same display precision (toFixed(1) per-type damage;
     integers for pools)
  C2 statistical: X = inspiration per-run values (R runs), Y = our compat MC per-run queue_dps_parity (n_runs ≥ 1000,
     seed 0); medians m_X, m_Y with distribution-free CI99 [X_(l), X_(u)], [Y_(l), Y_(u)] (E-8.19);
     pass iff [X_(l) − 0.01·m_X, X_(u) + 0.01·m_X] ∩ [Y_(l) − 0.01·m_Y, Y_(u) + 0.01·m_Y] ≠ ∅
     the same rule for avg per bullet (total), median per bullet (total) and the TTK median (+∞ allowed: both
     "not killed" passes)
  C3 game profile: C2 with our game-profile MC, only on scenarios that touch no register row
  validity: C1, C2 only when no touched row has Toggle "—"; C3 only when no row is touched
```

The median CI99 ranks are those of E-8.19 (`l = floor(R/2 − 1.28791465·√R)`, `u = ceil(1 + R/2 + 1.28791465·√R)`);
TV-08-16 is the known-answer test of the comparison function (`wfd-engine::mc::summary::oracle_compare`).

| Class | Profile | Runs | Valid when |
|---|---|---|---|
| C1 deterministic panels | compat | — | no touched row has Toggle `—` |
| C2 statistical | compat | ours n_runs ≥ 1000 (seed 0), inspiration R ≥ 200 | no touched row has Toggle `—` |
| C3 game profile | game | as C2 | `touched_rows` is empty |

Oracle-safety is decided automatically from the engine's trace (engine.md §10), never by hand. The scenario corpus
covers rifle, shotgun, pistol, bow, sniper, beam, launcher, melee with stance, Incarnon, kitgun, zaw, Kuva, exalted,
and every option of spec 00 §6 at least once; T36 records at least 30 oracle-safe scenarios.

### 5.3 In-game measurements

`research/test-cases/ingame/*.toml` (precedence 1 of spec 00 §1.4) are scenario vectors with the observed numbers and
their own measurement tolerance; they override oracle expectations when both exist, and a failing measurement opens a
register question rather than being "fixed" toward the oracle.

---

## 6. Optimizer

| Test | Rule |
|---|---|
| Brute-force exactness | on 100 random synthetic problems (≤ 18 candidates, 4 main slots, exilus, arcane, 2 shards), the argmax and the top-10 set equal full enumeration, `proof = ProvenOptimal` |
| Bound and monotonicity | the engine property tests of §2.3 run on optimizer-shaped inputs (partial `StatVec`s) |
| Keep-set rule | with an artificially perturbed `WindowFast` (error up to ε), the true optimum is still returned |
| Determinism | the same job with 1, 2 and 8 threads and a `node_budget`, repeated 5 times, returns identical results (batches with barrier thresholds and per-item budgets, optimizer.md §8); with a wall-clock limit only `threads = 1` is asserted reproducible |
| Certify | on the brute-force problems, `certify = true` gives `assumes_fast_error = None` and the same argmax as brute force |
| Constraints (proptest) | locked slots unchanged; must-include present; prohibited absent; no incompatible pair; at most one riven; capacity and Forma feasible; element rules respected; rule filters applied per effect |
| Cancellation | `DELETE` returns a result within 100 ms with a valid incumbent; time budget honoured |
| Beats WFOpt | `cargo xtask oracle-audit` (T40): on the oracle weapon set (about 15 weapons: rifle, shotgun, pistol, bow, beam, melee, Incarnon) with the inspiration-compatible preset, our best build scores at least WFOpt's recorded best (T36 recordings) under our `WindowExact` |
| Benchmarks | fixed rifle and melee problems with a node budget; nodes/s and evaluations/s reported (§9) |

---

## 7. Server

- Every route with `tower::ServiceExt::oneshot` against an `AppState` on the mini fixture bundle: status codes, JSON
  shapes (insta snapshots of responses), error bodies with message keys.
- SSE: event sequence (`started` → `progress`* → `done`), resume with `Last-Event-ID`, keep-alive, stream closed after
  the terminal event; cancel via `DELETE`.
- Security: wrong `Host` → 421, cross-origin `Origin` on POST → 403, body > 1 MB → 413.
- Superseded `/api/calc`: dropping the request cancels MC (cancel flag observed).
- CLI: `warframe-dps calc --input …`, `import <url>`, `optimize --node-budget` output snapshots.
- Startup: port fallback; second instance detects the first via `/api/meta` and exits 0.
- Data refresh: from a mocked source set (local HTTP mock), refresh produces the expected report and bundle;
  activation swaps data without restart; a running optimizer job keeps its snapshot.

---

## 8. Frontend

- **Vitest**: store actions and undo/redo; URL sync; baseline; `formatDamage` (U-5 rounding) and percent formatting;
  schema-driven renderers (a fixture schema with a new toggle renders without code changes); token contrast test over
  `tokens.css` (≥ 4.5:1 for text tokens on their surfaces); i18n key coverage (every used id exists, no unused ids).
- **MSW** mocks generated from `data/fixtures/mini` let component tests and early e2e work without the server.
- **Playwright** against `warframe-dps --no-browser --port 0 --bundle <mini bundle>` with a fixed MC seed and
  `images.fetch = false` (glyph tiles), so numbers and screenshots are stable and need no network:
  - parity suite: one spec per must-have row of frontend.md §6 (`PAR-nn` in the title);
  - keyboard-only flows: place, swap and remove a mod; build a riven; set an option; run and cancel the optimizer;
  - share code round trip; Furis inspiration-link import (`research/test-cases/user-example-furis.json`) with zero
    unmapped entries;
  - axe scan in every test (zero serious or critical violations);
  - visual: screenshots of the calculator, optimizer and data pages in dark and light at 375, 900 and 1440 px;
  - pseudo-locale run (`en-XA`) to detect hard-coded strings and truncation.

---

## 9. Performance

- `criterion` benches: `crates/wfd-engine/benches/{resolve,steady,window,bound,mc}.rs`,
  `crates/wfd-opt/benches/{bnb_rifle,bnb_melee}.rs`, `crates/wfd-data/benches/load.rs`.
- `cargo xtask bench-check` runs them in quick mode and fails when a median regresses by more than 25 % against
  `benches/baseline.json` or misses an absolute target of engine.md §13 by more than 2×.
- T37 records the measured numbers in `docs/architecture/performance.md`.

---

## 10. CI gates and commands

`cargo xtask ci` runs the gates in this order (each must pass before the next):

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask spec-sync --check && cargo xtask spec-vectors --check && cargo xtask gen-ts --check && cargo xtask lint-engine
cargo run -p wfd-datagen -- spec-tables --check && cargo run -p wfd-datagen -- check
cargo test --workspace
cargo xtask effect-smoke
npm --prefix frontend ci
npm --prefix frontend run lint && npm --prefix frontend run typecheck && npm --prefix frontend test -- --run
npm --prefix frontend run build
cargo build --release -p wfd-server
npx --prefix frontend playwright test
cargo xtask doc-check                       # every path, type and task id named in docs/architecture exists
```

`wfd-datagen check` needs the cache only for joins; in CI it runs after `wfd-datagen fetch --from-lock` (pipeline.md
§3.3).

Nightly (or on demand): proptest with 10k cases, EV-vs-MC with 10,000 runs, `cargo xtask bench-check`, the
cross-platform hash comparison, and `cargo xtask oracle-record --check` (re-records and diffs when the inspiration copy
is present). CLAUDE.md's rule ("`cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` must pass before a commit")
is the minimum for every commit.

---

## 11. Test data locations

| Path | Content |
|---|---|
| `data/fixtures/mini/` | hand-written bundle in the generated JSON format: Braton Prime, Furis + Incarnon line, a Kuva weapon, Paris, Acceltra (parent/child), a beam (Amprex), Arca Plasmor, a kitgun's parts, Nikana Prime with a stance, an exalted weapon, a sniper, a spool weapon; about 70 items covering every coverage row; Lancer, Corrupted Heavy Gunner, a shielded Corpus unit, an Eximus, an overguard unit, a boss with attenuation |
| `data/fixtures/scenarios/*.json` | `CalcInput`s used by spec vectors, golden tests and e2e (`braton_prime_lancer_l100_sp.json` …) |
| `data/fixtures/reference_weapons.toml` | reference weapon per item class for `xtask effect-smoke` (resolved by uniqueName in the generated bundle) |
| `crates/wfd-engine/tests/spec_vectors/*.toml` | machine twins of the spec vectors |
| `crates/wfd-engine/tests/golden/oracle/*.json` | recorded inspiration outputs |
| `research/test-cases/*.json` | user builds (inspiration links), e.g. `user-example-furis.json` |
| `research/test-cases/ingame/*.toml` | in-game measurements |
| `crates/wfd-buildcode/tests/codes/v1/*.txt` | sample share codes of every released version, kept forever |
| `tools/oracle/scenarios/*.json` | generated oracle scenario list |
