Status: draft (architecture)

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
| Spec vectors | every `TV-nn-mm` of `docs/mechanics/*.md` and the spec 00 §3.8 worked example | custom harness, probes | `crates/wfd-engine/tests/spec_vectors.rs`, `spec_conformance.rs`, `tests/spec_vectors/*.toml`; build-rule vectors (09) in `crates/wfd-data/tests/spec_vectors_09.rs` |
| Engine units | one test per formula or decision without a numeric vector (E-11 cases, CD-04 ordering, D-0xx decisions, NP-1/NP-2 f32 points) | `cargo test` | `crates/wfd-engine/src/**` `mod tests` |
| Coverage matrix | one fixture item per row of domain-model.md §17 with a hand-computed expected value | `cargo test` | `crates/wfd-engine/tests/effects_coverage.rs` |
| Mechanics | each mechanic kind in EV and MC on a minimal scenario | `cargo test` | `crates/wfd-engine/tests/mechanics.rs` |
| Properties | slot-permutation bit identity, incremental = full compile, monotonicity of `Inc` slots, `Bound ≥ WindowExact`, `WindowFast` error ≤ ε, quantization sum bounds, MC thread independence | proptest 1.12 (fixed seeds in CI, 10k cases nightly) | `crates/wfd-engine/tests/properties.rs` |
| EV vs MC | spec 00 §4.4 tolerance on golden scenarios | 1,000 runs in PR CI, 10,000 nightly | `crates/wfd-engine/tests/ev_vs_mc.rs` |
| Compat flags | each flag changes only its own register row's output | `cargo test` | `crates/wfd-engine/tests/compat.rs` |
| Effect smoke | every curated item reaches its declared stats, only inside its scope | `cargo test` | `crates/wfd-engine/tests/effect_smoke.rs` |
| Data pipeline | parsing, sandbox, join, validators, bundle, diff | `cargo test`, insta 1.49 | `crates/wfd-datagen/tests/`, `crates/wfd-data/tests/` |
| Oracle golden | inspiration run headless, seeded, compared with our compat and game profiles | Node + Playwright (recording), Rust (comparison) | `tools/oracle/`, `crates/wfd-engine/tests/golden_oracle.rs` |
| In-game measurements | `research/test-cases/ingame/*.toml` | `cargo test` | `crates/wfd-engine/tests/measured.rs` |
| Optimizer | exactness, bounds, determinism, constraints, cancellation, beats WFOpt | `cargo test`, proptest | `crates/wfd-opt/tests/` |
| Build codes | round trips, version samples, inspiration import | `cargo test`, proptest | `crates/wfd-buildcode/tests/` |
| Server | routes, SSE, security, CLI | `tokio::test`, `tower::ServiceExt::oneshot`, insta | `crates/wfd-server/tests/` |
| Frontend units | store, history, URL sync, formatters, schema-driven renderers, token contrast | Vitest 5.1 + Testing Library | `frontend/src/**/*.test.ts(x)` |
| End-to-end | one spec per parity row, keyboard flows, share/import, optimizer run and cancel | Playwright 1.65 (Chromium) | `frontend/e2e/` |
| Accessibility | axe on every page and open dialog | @axe-core/playwright 4.14 | every e2e test + `frontend/e2e/a11y.spec.ts` |
| Visual | screenshots of fixed scenarios, dark and light, 375 / 900 / 1440 px | Playwright `toHaveScreenshot` | `frontend/e2e/visual.spec.ts` |
| Performance | criterion benches, optimizer benchmark with node budget | criterion 0.8, `cargo xtask bench-check` | `crates/*/benches/` |
| Lints | fmt, clippy `-D warnings`, `xtask lint-engine`, `xtask spec-sync`, ESLint with i18n rule, `tsc --noEmit` | — | CI |

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
| Monotonicity | increasing any `Inc` slot never lowers `Steady`, `WindowFast` or `WindowExact` scores (failing seeds become fixtures) |
| Bound validity | for random partial builds and completions, `Kernel::bound(relaxed) ≥ WindowExact(completion)` |
| Fast error | `|WindowFast − WindowExact| / WindowExact ≤ ε` per weapon class (ε = 0.5 % unless the class is listed over budget) |
| Quantization | Σ q_t is within `n/64` of Σ r_t for n nonzero types |
| Determinism | `calculate` twice, and MC with rayon pools of 1 and 8 threads, give byte-identical serialized reports (I-5) |
| Cross-platform | the CI matrix (Linux, macOS, Windows) produces equal hashes of the reports of 10 fixed scenarios |

### 2.4 EV vs MC

On the golden scenario corpus (§11), EV `dps_window` must lie inside the MC 99 % CI widened by 2 % (spec 00 §4.4),
except scenarios tagged with an approximation class of engine.md §7.6 (then the expected gap is asserted instead).

### 2.5 Lints

- `cargo xtask lint-engine`: no float literals outside `spec_constants.rs` and tests in `wfd-engine` and `wfd-opt`
  (allow-list 0, 1, 2, 0.5) (I-1); no `/Lotus/` and no item name of the mini fixture in their sources (I-2);
  `clippy::disallowed_types` forbids `HashMap`/`HashSet` in result paths.
- `cargo xtask spec-sync`: `CompatFlags` ↔ register rows marked `Compat`; `TOGGLE_MAP` ↔ spec 00 §6.1; list of
  `SPEC-PENDING` markers.

---

## 3. Effects and curated data

**Effect smoke test** (`crates/wfd-engine/tests/effect_smoke.rs`): for every curated item in the committed data,
pick its reference scenario (a weapon of the item's class from `data/fixtures/reference_weapons.toml`, a fixed enemy),
evaluate with and without the item (and with each conditional effect forced on), and assert that every declared target
changes in the declared direction on the attacks and hits in its scope and does not change outside the scope. This
catches mis-scoped effects that would silently do nothing. Run per item with `-- <slug>`; the full run is part of CI.

**Validators** (pipeline.md §7): each check has one passing and one failing fixture under
`crates/wfd-datagen/tests/fixtures/curated/`. `wfd-datagen check` on the committed `data/curated/` is a CI gate.

---

## 4. Data pipeline

- **No network in tests.** Wire parsing runs on trimmed upstream excerpts committed under
  `crates/wfd-datagen/tests/fixtures/upstream/` (small, with licence notes); the fetcher is tested against a local
  HTTP mock (ETag, revid, 429 with `Retry-After`, 404 caching, cancellation).
- **Lua sandbox**: no `io`/`os`/`package`/`load`; instruction and memory limits trip on hostile fixture modules;
  every cached module converts (an ignored test runs over the full cache locally).
- **Typos and unknown fields**: alias fixtures; unknown fields counted in the report.
- **Join thresholds**: WFCD→PE 668/668 weapons; wiki weapons→PE ≥ 825/872; wiki mods→PE ≥ 1,451/1,635 (asserted when
  the full cache is present; `#[ignore]` otherwise, run by T18 and in the data-update workflow).
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
  ≥ 200 seeds per scenario and writes `crates/wfd-engine/tests/golden/oracle/<case>.json` with the numbers only, the
  inspiration bundle SHA-256, the date and the seeds. Recording is a manual step because the inspiration copy is not
  in the repository; the recordings are committed and the comparison runs in normal CI.

### 5.2 Comparison classes

| Class | Profile | What is compared | Pass rule | Valid when |
|---|---|---|---|---|
| 1. Deterministic panels | compat | modded per-type damage, crit, status, fire rate of the weapon panel; scaled health, shield, armor, overguard, DR of the enemy panel | equal to the inspiration's display precision | always (deterministic) |
| 2. Statistical | compat | `queue_dps_parity` and the per-bullet outputs: median over our ≥ 2,000 seeded compat MC runs vs the median over the ≥ 200 recorded inspiration runs | medians agree within the bootstrap 99 % CI of the difference plus the 08 tolerance (SPEC-PENDING(08); provisional 2 %) | `touched_rows` contains no register row whose Toggle column is `—` |
| 3. Game profile | game | same as class 2 with the normal profile | same | `touched_rows` is empty (no register row applies at all) |

Oracle-safety is decided automatically from the engine's trace (engine.md §10), never by hand. The scenario corpus
covers rifle, shotgun, pistol, bow, sniper, beam, launcher, melee with stance, Incarnon, kitgun, zaw, Kuva, exalted,
and every option of spec 00 §6 at least once; T32 records at least 30 oracle-safe scenarios.

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
| Determinism | the same job with 1, 2 and 8 threads and a `node_budget`, repeated 5 times, returns identical results |
| Constraints (proptest) | locked slots unchanged; must-include present; prohibited absent; no incompatible pair; at most one riven; capacity and Forma feasible; element rules respected; rule filters applied per effect |
| Cancellation | `DELETE` returns a result within 100 ms with a valid incumbent; time budget honoured |
| Beats WFOpt | on the oracle weapon set (about 15 weapons: rifle, shotgun, pistol, bow, beam, melee, Incarnon) with the inspiration-compatible preset, our best build scores at least WFOpt's recorded best under our `WindowExact` |
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
- **Playwright** against `warframe-dps --no-browser --port 0 --bundle <mini bundle>` with a fixed MC seed, so numbers
  are stable:
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
- T33 records the measured numbers in `docs/architecture/performance.md`.

---

## 10. CI gates and commands

`cargo xtask ci` runs the gates in this order (each must pass before the next):

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo xtask spec-sync --check && cargo xtask spec-vectors --check && cargo xtask gen-ts --check && cargo xtask lint-engine
cargo run -p wfd-datagen -- check
cargo test --workspace
npm --prefix frontend ci
npm --prefix frontend run lint && npm --prefix frontend run typecheck && npm --prefix frontend test -- --run
npm --prefix frontend run build
cargo build --release -p wfd-server
npx --prefix frontend playwright test
```

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
| `data/fixtures/reference_weapons.toml` | reference weapon per item class for effect smoke tests |
| `crates/wfd-engine/tests/spec_vectors/*.toml` | machine twins of the spec vectors |
| `crates/wfd-engine/tests/golden/oracle/*.json` | recorded inspiration outputs |
| `research/test-cases/*.json` | user builds (inspiration links), e.g. `user-example-furis.json` |
| `research/test-cases/ingame/*.toml` | in-game measurements |
| `crates/wfd-buildcode/tests/codes/v1/*.txt` | sample share codes of every released version, kept forever |
| `tools/oracle/scenarios/*.json` | generated oracle scenario list |
