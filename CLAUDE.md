# warframe-dps

A local Warframe DPS calculator with an automated build optimizer. Rust backend (calculation engine, optimizer, local HTTP server); browser frontend served by that server. Goal: match or exceed the feature set of warframe-damage.com, with damage numbers as close to the game as research allows.

## Layout

- `docs/` — finished, reviewed documentation. Treat it as the source of truth.
  - `docs/mechanics/` — the damage-calculation spec. Code must follow it; if code and spec disagree, one of them is a bug.
  - `docs/architecture/` — crate layout, data model, API, frontend design.
  - `docs/data/` — where game data comes from and how it is built.
- `research/` — unfinished findings. Anything here may be wrong. Promote to `docs/` only after it has been cross-checked.
  - `research/findings/` — one Markdown file per topic, each starting with a `Status:` line (draft / cross-checked / superseded).
  - `research/test-cases/` — builds with known expected results, used for golden tests.
  - `research/external/` — third-party reference material (gitignored, never commit it). `inspiration/min.pretty.js` is the prettified warframe-damage.com bundle: all of its data, simulation and optimizer code.

## Conventions

- Cite a source for every damage-mechanic claim: a wiki.warframe.com page, a line range in `research/external/inspiration/min.pretty.js`, a patch note, or a test. Mark anything unsourced as `UNVERIFIED`.
- When the inspiration and the wiki disagree, record both and the reason for the choice in the relevant `docs/mechanics/` file; do not silently pick one.
- Game data is built from open sources (WFCD warframe-items, DE Public Export, wiki.warframe.com data modules). The inspiration bundle is a reference and a test oracle, not a data source to copy.
- Rust: stable toolchain, `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` must pass before a commit.

## Standing corrections

Mistakes that have happened more than once, and the rule that prevents them. Read these before working; the manager adds to this list.

(none yet)
