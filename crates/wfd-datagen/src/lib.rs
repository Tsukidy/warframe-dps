//! `wfd-datagen`: the data pipeline of warframe-dps (docs/data/pipeline.md), as a library (the server's in-app
//! refresh calls it behind its feature `refresh`) and as the CLI `wfd-datagen` (`src/main.rs`).
//!
//! The pipeline reads three upstream sources (wiki Lua modules, DE Public Export, WFCD warframe-items) plus the spec
//! tables and the curated TOML, and writes the canonical JSON of `data/generated/` and the bundle (pipeline.md §2):
//!
//! | Stage | Module | Owner (work-breakdown.md §8) |
//! |---|---|---|
//! | fetch: network → `.cache/wfd-sources/` (the only stage that touches the network) | [`mod@fetch`] | T05 |
//! | parse: cache → typed wire records (mlua sandbox for wiki modules) | [`lua`], [`wiki`], [`pe`], [`wfcd`] | T05 |
//! | join: wire records → one catalogue keyed by uniqueName (§5.1) | [`mod@join`] | T18 |
//! | derive: attack keys and roles, parents, triggers, compatibility, stances, heavies, frames, stable keys (§5.3) | [`mod@derive`] | T18 |
//! | spec tables: tier-2 inputs from `docs/mechanics/` (§5.2) | [`spec_tables`] | T18 |
//! | effects: tier 1 stat lines, tier 2 spec tables, tier 3 curated TOML (§6) | [`effects`], [`curated`] | T19 |
//! | validate: the checks of §7 | [`mod@validate`] | T19 |
//! | emit: `data/generated/*.json`, `REPORT.md`, `report.json`, bundle (§8) | [`emit`], [`report`], [`render`] | T18 (render T19) |
//!
//! This file and `config.rs` ([`PipelineConfig`], [`InputTree`], [`SpecSource`], [`LockMode`]), `error.rs`
//! ([`DatagenError`]), `progress.rs` ([`Progress`]) and [`cli`] are the frozen library API of T02c (work-breakdown.md
//! §2.1). The entry points [`fetch()`], [`build()`] and [`diff()`] check cancellation and delegate to the module of the
//! implementing task, so no later task edits this file; a change to these signatures, and a new field of
//! [`PipelineConfig`] or a new [`DatagenError`] variant, is a `CONTRACTS.md` entry. The record types the stages pass
//! between each other ([`WikiRecords`], [`Joined`], [`SpecTables`], …) are created empty: their owners define the
//! fields.
//!
//! The CLI subcommands live with their implementing task too: `fetch` in [`fetch::cli`] (T05), `curate` in
//! [`curated::cli`] (T19), the others in `main.rs` (T18); the shared options are [`cli::CommonArgs`].

use std::sync::atomic::AtomicBool;

pub mod cli;
mod config;
mod error;
mod progress;

pub mod fetch;
pub mod lua;
pub mod pe;
pub mod wfcd;
pub mod wiki;

pub mod derive;
pub mod emit;
pub mod join;
pub mod report;
pub mod spec_tables;

pub mod curated;
pub mod effects;
pub mod render;
pub mod validate;

pub use config::{
    CACHE_DIR, CURATED_DIR, GENERATED_DIR, InputTree, LOCK_FILE, LockMode, PipelineConfig,
    SPEC_TABLES_FILE, SourceKind, SpecSource, VALIDATION_DIR,
};
pub use error::DatagenError;
pub use progress::{NoProgress, PipelineStep, Progress, check_cancel};

pub use derive::{heavy_def, normalise_stances};
pub use effects::{EffectInputs, attach_effects};
pub use emit::{BuildOutput, GeneratedJson};
pub use fetch::{FetchReport, fetch_from_lock};
pub use join::{Joined, JoinedWeapon, join};
pub use lua::{LuaLimits, eval_wiki_modules};
pub use pe::PeRecords;
pub use report::{DataDiff, Report};
pub use spec_tables::{SpecTables, SpecTablesConfig, ingest_spec_tables};
pub use validate::{ValidationConfig, validate};
pub use wfcd::WfcdRecords;
pub use wiki::{WikiRecords, WikiSource};

/// Fills the source cache (pipeline.md §3): with [`LockMode::FromLock`] exactly the inputs of the lock file
/// ([`fetch_from_lock`], §3.3), otherwise the newest inputs ([`LockMode::Latest`], writing the lock when
/// [`PipelineConfig::update_lock`] is set) or the pinned ones ([`LockMode::Pinned`]). Cancellation is checked before
/// the first request and between requests. Implemented by T05 ([`fetch::fetch_sources`]).
pub fn fetch(
    cfg: &PipelineConfig,
    progress: &dyn Progress,
    cancel: &AtomicBool,
) -> Result<FetchReport, DatagenError> {
    check_cancel(cancel)?;
    match cfg.lock {
        LockMode::FromLock => fetch::fetch_from_lock(cfg, progress, cancel),
        LockMode::Pinned | LockMode::Latest => fetch::fetch_sources(cfg, progress, cancel),
    }
}

/// Runs parse → join → derive → effects → validate → emit (pipeline.md §2) and returns the bundle, its canonical JSON
/// and the report; writing them is the caller's choice (the CLI writes `data/generated/`, the server a user bundle).
/// Unless [`PipelineConfig::offline`] is set, it fetches the inputs of [`PipelineConfig::lock`] first. Validation
/// errors fail the build with [`DatagenError::Validation`], which carries the report. Implemented by T18
/// ([`emit::run_build`]); the effects and validation stages are T19's.
pub fn build(
    cfg: &PipelineConfig,
    progress: &dyn Progress,
    cancel: &AtomicBool,
) -> Result<BuildOutput, DatagenError> {
    check_cancel(cancel)?;
    emit::run_build(cfg, progress, cancel)
}

/// Field-level difference of two bundles ("Furis/normal_attack cc 0.05 → 0.07", pipeline.md §2), for
/// `wfd-datagen diff`, the developer refresh (§9.1) and the in-app refresh report (§9.2, [`DataDiff::summary`]).
/// Implemented by T18 ([`report::diff_bundles`]).
pub fn diff(old: &wfd_model::Bundle, new: &wfd_model::Bundle) -> DataDiff {
    report::diff_bundles(old, new)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    /// `docs/data/pipeline.md`, which the doc-tied tests of the modules read (the constants and synopses there are
    /// the contract).
    pub(crate) fn pipeline_md() -> String {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/data/pipeline.md");
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// The second cell of the Markdown table row whose first cell is `first` (exactly one such row).
    pub(crate) fn table_cell(md: &str, first: &str) -> String {
        let rows: Vec<&str> = md
            .lines()
            .filter(|l| l.split('|').nth(1).map(str::trim) == Some(first))
            .collect();
        assert_eq!(rows.len(), 1, "one table row `| {first} |` expected");
        rows[0].split('|').nth(2).unwrap().trim().to_owned()
    }

    /// The code spans of a Markdown text, with `a/{b,c}` expanded to `a/b`, `a/c`.
    pub(crate) fn code_spans(text: &str) -> Vec<String> {
        let mut out = Vec::new();
        for span in text.split('`').skip(1).step_by(2) {
            match (span.find('{'), span.find('}')) {
                (Some(o), Some(c)) if o < c => {
                    for alt in span[o + 1..c].split(',') {
                        out.push(format!("{}{}{}", &span[..o], alt, &span[c + 1..]));
                    }
                }
                _ => out.push(span.to_owned()),
            }
        }
        out
    }

    #[test]
    fn code_span_expansion() {
        assert_eq!(code_spans("x `a/{b,c}` y `d` z"), ["a/b", "a/c", "d"]);
    }

    /// A cancelled job never reaches the network or the build (pipeline.md §3.1 "Cancellation").
    #[test]
    fn entry_points_check_cancellation_first() {
        let cancel = AtomicBool::new(true);
        for lock in [LockMode::Pinned, LockMode::Latest, LockMode::FromLock] {
            let mut cfg = PipelineConfig::developer(std::path::Path::new("."));
            cfg.lock = lock;
            assert!(matches!(
                fetch(&cfg, &NoProgress, &cancel),
                Err(DatagenError::Cancelled)
            ));
            assert!(matches!(
                build(&cfg, &NoProgress, &cancel),
                Err(DatagenError::Cancelled)
            ));
        }
    }
}
